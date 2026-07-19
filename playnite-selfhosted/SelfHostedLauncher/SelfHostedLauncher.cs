using Playnite.SDK;
using Playnite.SDK.Events;
using Playnite.SDK.Models;
using Playnite.SDK.Plugins;
using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using Playnite.SDK.Data;

namespace SelfHostedLauncher
{
    public class SelfHostedLauncher : GenericPlugin
    {
        private static readonly ILogger logger = LogManager.GetLogger();
        private SelfHostedLauncherSettingsViewModel settings { get; set; }

        public override Guid Id { get; } = Guid.Parse("c488347c-21a4-4f81-a67b-bfeb21e0589a");

        public SelfHostedLauncher(IPlayniteAPI api) : base(api)
        {
            settings = new SelfHostedLauncherSettingsViewModel(this);
            Properties = new GenericPluginProperties
            {
                HasSettings = true
            };
        }

        public override ISettings GetSettings(bool firstRunSettings)
        {
            return settings;
        }

        public override System.Windows.Controls.UserControl GetSettingsView(bool firstRunSettings)
        {
            // Simple placeholder configuration screen. 
            // In a full implementation, this could return a WPF XAML UserControl.
            return null;
        }

        public override IEnumerable<GameMenuItem> GetGameMenuItems(GetGameMenuItemsArgs args)
        {
            yield return new GameMenuItem
            {
                MenuSection = "Self-Hosted Cloud Sync",
                Description = "Загрузить сборку в облако",
                Action = async (menuArgs) =>
                {
                    var game = menuArgs.Games.FirstOrDefault();
                    if (game != null)
                    {
                        await UploadGameBuildAsync(game);
                    }
                }
            };

            yield return new GameMenuItem
            {
                MenuSection = "Self-Hosted Cloud Sync",
                Description = "Установить сборку из облака",
                Action = async (menuArgs) =>
                {
                    var game = menuArgs.Games.FirstOrDefault();
                    if (game != null)
                    {
                        await DownloadGameBuildAsync(game);
                    }
                }
            };

            yield return new GameMenuItem
            {
                MenuSection = "Self-Hosted Cloud Sync",
                Description = "Выгрузить сейвы в облако (вручную)",
                Action = async (menuArgs) =>
                {
                    var game = menuArgs.Games.FirstOrDefault();
                    if (game != null)
                    {
                        await UploadSavesAsync(game);
                    }
                }
            };

            yield return new GameMenuItem
            {
                MenuSection = "Self-Hosted Cloud Sync",
                Description = "Скачать сейвы из облака (вручную)",
                Action = async (menuArgs) =>
                {
                    var game = menuArgs.Games.FirstOrDefault();
                    if (game != null)
                    {
                        await DownloadSavesAsync(game);
                    }
                }
            };
        }

        public override async void OnGameStarting(OnGameStartingEventArgs args)
        {
            logger.Info($"Игра {args.Game.Name} запускается. Скачиваем сейвы...");
            await DownloadSavesAsync(args.Game);
        }

        public override async void OnGameStopped(OnGameStoppedEventArgs args)
        {
            logger.Info($"Игра {args.Game.Name} закрыта. Синхронизируем прогресс...");
            
            // 1. Log playtime
            await LogPlaytimeAsync(args.Game, args.ElapsedSeconds);

            // 2. Upload saves
            await UploadSavesAsync(args.Game);

            // 3. Sync achievements (if Goldberg Steam Emulator is used)
            await SyncGoldbergAchievementsAsync(args.Game);
        }

        #region API Helpers

        private HttpClient GetAuthenticatedClient()
        {
            var client = new HttpClient();
            client.BaseAddress = new Uri(settings.Settings.ServerUrl.TrimEnd('/') + "/");
            if (!string.IsNullOrEmpty(settings.Settings.Token))
            {
                client.DefaultRequestHeaders.Authorization = new AuthenticationHeaderValue("Bearer", settings.Settings.Token);
            }
            return client;
        }

        #endregion

        #region Game Builds (CDN)

        private async Task UploadGameBuildAsync(Game game)
        {
            if (string.IsNullOrEmpty(game.InstallDirectory) || !Directory.Exists(game.InstallDirectory))
            {
                PlayniteApi.Dialogs.ShowErrorMessage("Игра должна быть установлена локально, чтобы загрузить сборку в облако.", "Ошибка");
                return;
            }

            var progressOptions = new GlobalProgressOptions("Создание архива и загрузка сборки в облако...", true)
            {
                IsIndeterminate = false
            };

            PlayniteApi.Dialogs.ActivateGlobalProgress(async (progressContext) =>
            {
                try
                {
                    progressContext.Text = "Архивирование папки с игрой...";
                    progressContext.CurrentProgressValue = 10;
                    
                    var tempZip = Path.Combine(Path.GetTempPath(), $"{game.Id}.7z");
                    if (File.Exists(tempZip)) File.Delete(tempZip);

                    // For simplicity using ZipFile, in production a 7-Zip process is preferred for size
                    await Task.Run(() => ZipFile.CreateFromDirectory(game.InstallDirectory, tempZip, CompressionLevel.Optimal, false));

                    progressContext.Text = "Регистрация сборки на сервере...";
                    progressContext.CurrentProgressValue = 50;

                    using (var client = GetAuthenticatedClient())
                    {
                        // Register game on backend first
                        var regContent = new StringContent(
                            Serialization.ToJson(new { playnite_id = game.Id.ToString(), name = game.Name }),
                            Encoding.UTF8, "application/json"
                        );
                        await client.PostAsync("games/", regContent);

                        // Request S3 presigned upload URL
                        var response = await client.PostAsync($"games/{game.Id}/upload-url?extension=7z", null);
                        if (!response.IsSuccessStatusCode)
                        {
                            throw new Exception("Не удалось получить ссылку для загрузки.");
                        }

                        var json = await response.Content.ReadAsStringAsync();
                        var uploadInfo = Serialization.FromJson<UploadInfo>(json);

                        progressContext.Text = "Отправка архива в облако MinIO...";
                        progressContext.CurrentProgressValue = 70;

                        // Upload file to S3 presigned URL
                        using (var fileStream = File.OpenRead(tempZip))
                        using (var uploadClient = new HttpClient())
                        {
                            uploadClient.Timeout = TimeSpan.FromHours(2); // Large file timeout
                            var content = new StreamContent(fileStream);
                            var s3Response = await uploadClient.PutAsync(uploadInfo.upload_url, content);
                            if (!s3Response.IsSuccessStatusCode)
                            {
                                throw new Exception("Сбой отправки файла в S3.");
                            }
                        }

                        // Confirm upload to backend
                        await client.PostAsync($"games/{game.Id}/confirm-upload", null);
                    }

                    if (File.Exists(tempZip)) File.Delete(tempZip);
                    
                    progressContext.CurrentProgressValue = 100;
                    PlayniteApi.Dialogs.ShowMessage($"Сборка игры {game.Name} успешно сохранена в облаке!", "Успех");
                }
                catch (Exception ex)
                {
                    logger.Error(ex, "Failed to upload game build");
                    PlayniteApi.Dialogs.ShowErrorMessage($"Ошибка при загрузке сборки: {ex.Message}", "Ошибка");
                }
            }, progressOptions);
        }

        private async Task DownloadGameBuildAsync(Game game)
        {
            var selectPath = PlayniteApi.Dialogs.SelectFolder();
            if (string.IsNullOrEmpty(selectPath)) return;

            var progressOptions = new GlobalProgressOptions("Скачивание сборки из облака...", true)
            {
                IsIndeterminate = false
            };

            PlayniteApi.Dialogs.ActivateGlobalProgress(async (progressContext) =>
            {
                try
                {
                    progressContext.Text = "Получение ссылки на скачивание...";
                    progressContext.CurrentProgressValue = 10;

                    using (var client = GetAuthenticatedClient())
                    {
                        var response = await client.GetAsync($"games/{game.Id}/download-url");
                        if (!response.IsSuccessStatusCode)
                        {
                            throw new Exception("Сборка не найдена в облаке.");
                        }

                        var json = await response.Content.ReadAsStringAsync();
                        var downloadInfo = Serialization.FromJson<DownloadInfo>(json);

                        progressContext.Text = "Загрузка архива сборки...";
                        progressContext.CurrentProgressValue = 30;

                        var tempZip = Path.Combine(Path.GetTempPath(), $"{game.Id}.7z");
                        if (File.Exists(tempZip)) File.Delete(tempZip);

                        using (var downloadClient = new HttpClient())
                        using (var s3Stream = await downloadClient.GetStreamAsync(downloadInfo.download_url))
                        using (var fileStream = File.Create(tempZip))
                        {
                            await s3Stream.CopyToAsync(fileStream);
                        }

                        progressContext.Text = "Распаковка файлов игры...";
                        progressContext.CurrentProgressValue = 70;

                        var destDir = Path.Combine(selectPath, game.Name);
                        if (Directory.Exists(destDir)) Directory.Delete(destDir, true);
                        Directory.CreateDirectory(destDir);

                        await Task.Run(() => ZipFile.ExtractToDirectory(tempZip, destDir));

                        if (File.Exists(tempZip)) File.Delete(tempZip);

                        // Find executable to configure play action
                        var exeFiles = Directory.GetFiles(destDir, "*.exe", SearchOption.AllDirectories);
                        var mainExe = exeFiles.OrderBy(f => new FileInfo(f).Length).LastOrDefault(); // heuristically largest exe

                        game.InstallDirectory = destDir;
                        game.IsInstalled = true;
                        
                        if (mainExe != null)
                        {
                            game.GameActions = new System.Collections.ObjectModel.ObservableCollection<GameAction>
                            {
                                new GameAction
                                {
                                    Name = "Play (Cloud Build)",
                                    Type = GameActionType.File,
                                    Path = mainExe,
                                    WorkingDir = Path.GetDirectoryName(mainExe),
                                    IsPlayAction = true
                                }
                            };
                        }

                        PlayniteApi.Database.Games.Update(game);
                    }

                    progressContext.CurrentProgressValue = 100;
                    PlayniteApi.Dialogs.ShowMessage($"Игра {game.Name} успешно установлена и готова к запуску!", "Успех");
                }
                catch (Exception ex)
                {
                    logger.Error(ex, "Failed to download game build");
                    PlayniteApi.Dialogs.ShowErrorMessage($"Ошибка при установке игры: {ex.Message}", "Ошибка");
                }
            }, progressOptions);
        }

        #endregion

        #region Cloud Saves

        private async Task UploadSavesAsync(Game game)
        {
            // Look up local save path config
            var savePath = GetSaveDirectory(game);
            if (string.IsNullOrEmpty(savePath) || !Directory.Exists(savePath))
            {
                // Prompt user to select save folder if not defined
                var selected = PlayniteApi.Dialogs.SelectFolder();
                if (string.IsNullOrEmpty(selected)) return;
                savePath = selected;
                SaveSaveDirectory(game, savePath);
            }

            try
            {
                var tempZip = Path.Combine(Path.GetTempPath(), $"{game.Id}_saves.zip");
                if (File.Exists(tempZip)) File.Delete(tempZip);

                ZipFile.CreateFromDirectory(savePath, tempZip);

                using (var client = GetAuthenticatedClient())
                {
                    var response = await client.PostAsync($"saves/{game.Id}/upload-url", null);
                    if (!response.IsSuccessStatusCode) return;

                    var json = await response.Content.ReadAsStringAsync();
                    var uploadInfo = Serialization.FromJson<UploadInfo>(json);

                    using (var fileStream = File.OpenRead(tempZip))
                    using (var uploadClient = new HttpClient())
                    {
                        var content = new StreamContent(fileStream);
                        await uploadClient.PutAsync(uploadInfo.upload_url, content);
                    }
                }

                if (File.Exists(tempZip)) File.Delete(tempZip);
                logger.Info($"Сейвы игры {game.Name} успешно выгружены в облако.");
            }
            catch (Exception ex)
            {
                logger.Error(ex, "Failed to upload saves");
            }
        }

        private async Task DownloadSavesAsync(Game game)
        {
            var savePath = GetSaveDirectory(game);
            if (string.IsNullOrEmpty(savePath)) return;

            try
            {
                using (var client = GetAuthenticatedClient())
                {
                    var response = await client.GetAsync($"saves/{game.Id}/download-url");
                    if (!response.IsSuccessStatusCode)
                    {
                        logger.Info("В облаке нет сохранений для этой игры.");
                        return;
                    }

                    var json = await response.Content.ReadAsStringAsync();
                    var downloadInfo = Serialization.FromJson<DownloadInfo>(json);

                    var tempZip = Path.Combine(Path.GetTempPath(), $"{game.Id}_saves.zip");
                    if (File.Exists(tempZip)) File.Delete(tempZip);

                    using (var downloadClient = new HttpClient())
                    using (var s3Stream = await downloadClient.GetStreamAsync(downloadInfo.download_url))
                    using (var fileStream = File.Create(tempZip))
                    {
                        await s3Stream.CopyToAsync(fileStream);
                    }

                    if (!Directory.Exists(savePath)) Directory.CreateDirectory(savePath);
                    
                    // Backup local saves just in case
                    var backupLocal = savePath + "_local_backup";
                    if (Directory.Exists(backupLocal)) Directory.Delete(backupLocal, true);
                    Directory.Move(savePath, backupLocal);
                    Directory.CreateDirectory(savePath);

                    try
                    {
                        ZipFile.ExtractToDirectory(tempZip, savePath);
                    }
                    catch
                    {
                        // Rollback on fail
                        Directory.Delete(savePath, true);
                        Directory.Move(backupLocal, savePath);
                        throw;
                    }

                    if (Directory.Exists(backupLocal)) Directory.Delete(backupLocal, true);
                    if (File.Exists(tempZip)) File.Delete(tempZip);
                    logger.Info("Облачные сохранения успешно восстановлены.");
                }
            }
            catch (Exception ex)
            {
                logger.Error(ex, "Failed to download saves");
            }
        }

        private string GetSaveDirectory(Game game)
        {
            // Simple helper: stores save paths locally in plugin user data config
            var configPath = Path.Combine(GetPluginUserDataPath(), $"{game.Id}_savepath.txt");
            if (File.Exists(configPath))
            {
                return File.ReadAllText(configPath);
            }
            return null;
        }

        private void SaveSaveDirectory(Game game, string path)
        {
            var configPath = Path.Combine(GetPluginUserDataPath(), $"{game.Id}_savepath.txt");
            Directory.CreateDirectory(GetPluginUserDataPath());
            File.WriteAllText(configPath, path);
        }

        #endregion

        #region Playtime & Achievements

        private async Task LogPlaytimeAsync(Game game, ulong elapsedSeconds)
        {
            if (elapsedSeconds < 10) return; // Ignore very short sessions
            try
            {
                using (var client = GetAuthenticatedClient())
                {
                    var content = new StringContent(
                        Serialization.ToJson(new { playnite_id = game.Id.ToString(), duration_seconds = (int)elapsedSeconds }),
                        Encoding.UTF8, "application/json"
                    );
                    await client.PostAsync("playtime/log", content);
                }
            }
            catch (Exception ex)
            {
                logger.Error(ex, "Failed to log playtime");
            }
        }

        private async Task SyncGoldbergAchievementsAsync(Game game)
        {
            // Locate Steam App ID from game source/tags or actions
            string steamAppId = null;
            if (game.Source?.Name == "Steam")
            {
                steamAppId = game.GameId; // Playnite Steam game ID is the App ID
            }
            else
            {
                // Fallback: search game actions for steam url containing app ID or parse manually
                foreach (var action in game.GameActions ?? Enumerable.Empty<GameAction>())
                {
                    if (action.Path != null && action.Path.Contains("steam://run/"))
                    {
                        steamAppId = action.Path.Replace("steam://run/", "").Trim();
                        break;
                    }
                }
            }

            if (string.IsNullOrEmpty(steamAppId)) return;

            // Path to Goldberg achievements file: %APPDATA%\Goldberg SteamEmu Saves\<AppId>\achievements.json
            var appData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
            var achievementsFile = Path.Combine(appData, "Goldberg SteamEmu Saves", steamAppId, "achievements.json");

            if (!File.Exists(achievementsFile)) return;

            try
            {
                var json = File.ReadAllText(achievementsFile);
                var goldbergAchs = Serialization.FromJson<Dictionary<string, GoldbergAchievement>>(json);
                if (goldbergAchs == null) return;

                var listToSync = new List<object>();
                foreach (var kp in goldbergAchs)
                {
                    listToSync.Add(new
                    {
                        playnite_id = game.Id.ToString(),
                        api_name = kp.Key,
                        name = kp.Key, // Fallback display name
                        unlocked = kp.Value.unlocked == 1,
                        unlock_time = kp.Value.unlock_time > 0 
                            ? DateTimeOffset.FromUnixTimeSeconds(kp.Value.unlock_time).UtcDateTime 
                            : (DateTime?)null
                    });
                }

                using (var client = GetAuthenticatedClient())
                {
                    var content = new StringContent(
                        Serialization.ToJson(listToSync),
                        Encoding.UTF8, "application/json"
                    );
                    await client.PostAsync($"achievements/{game.Id}/sync", content);
                }
                logger.Info("Локальные достижения эмулятора Goldberg синхронизированы с облаком.");
            }
            catch (Exception ex)
            {
                logger.Error(ex, "Failed to sync achievements");
            }
        }

        private class GoldbergAchievement
        {
            public int unlocked { get; set; }
            public long unlock_time { get; set; }
        }

        #endregion

        #region Serialized Helper Classes

        private class UploadInfo
        {
            public string upload_url { get; set; }
            public string s3_key { get; set; }
        }

        private class DownloadInfo
        {
            public string download_url { get; set; }
        }

        #endregion
    }
}
