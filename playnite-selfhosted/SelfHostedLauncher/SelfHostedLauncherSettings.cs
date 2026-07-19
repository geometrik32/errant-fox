using Playnite.SDK;
using Playnite.SDK.Data;
using System;
using System.Collections.Generic;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Text;
using System.Threading.Tasks;

namespace SelfHostedLauncher
{
    public class SelfHostedLauncherSettings : ObservableObject
    {
        private string serverUrl = "http://localhost:8000";
        public string ServerUrl
        {
            get => serverUrl;
            set => SetValue(ref serverUrl, value);
        }

        private string username = "";
        public string Username
        {
            get => username;
            set => SetValue(ref username, value);
        }

        private string password = "";
        public string Password
        {
            get => password;
            set => SetValue(ref password, value);
        }

        private string token = "";
        public string Token
        {
            get => token;
            set => SetValue(ref token, value);
        }
    }

    public class SelfHostedLauncherSettingsViewModel : ObservableObject, ISettings
    {
        private readonly SelfHostedLauncher plugin;
        private SelfHostedLauncherSettings editingClone;

        private SelfHostedLauncherSettings settings;
        public SelfHostedLauncherSettings Settings
        {
            get => settings;
            set => SetValue(ref settings, value);
        }

        public SelfHostedLauncherSettingsViewModel(SelfHostedLauncher plugin)
        {
            this.plugin = plugin;
            var savedSettings = plugin.LoadPluginSettings<SelfHostedLauncherSettings>();
            if (savedSettings != null)
            {
                Settings = savedSettings;
            }
            else
            {
                Settings = new SelfHostedLauncherSettings();
            }
        }

        public void BeginEdit()
        {
            editingClone = Serialization.GetClone(Settings);
        }

        public void CancelEdit()
        {
            Settings = editingClone;
        }

        public void EndEdit()
        {
            plugin.SavePluginSettings(Settings);
        }

        public bool VerifySettings(out List<string> errors)
        {
            errors = new List<string>();
            if (string.IsNullOrWhiteSpace(Settings.ServerUrl))
            {
                errors.Add("URL сервера бэкенда не может быть пустым.");
            }
            return errors.Count == 0;
        }

        public async Task<bool> AuthenticateAsync()
        {
            if (string.IsNullOrWhiteSpace(Settings.ServerUrl) || 
                string.IsNullOrWhiteSpace(Settings.Username) || 
                string.IsNullOrWhiteSpace(Settings.Password))
            {
                return false;
            }

            try
            {
                using (var client = new HttpClient())
                {
                    var content = new FormUrlEncodedContent(new[]
                    {
                        new KeyValuePair<string, string>("username", Settings.Username),
                        new KeyValuePair<string, string>("password", Settings.Password)
                    });

                    var response = await client.PostAsync($"{Settings.ServerUrl.TrimEnd('/')}/auth/token", content);
                    if (response.IsSuccessStatusCode)
                    {
                        var json = await response.Content.ReadAsStringAsync();
                        var tokenResponse = Serialization.FromJson<TokenData>(json);
                        Settings.Token = tokenResponse.access_token;
                        plugin.SavePluginSettings(Settings);
                        return true;
                    }
                }
            }
            catch (Exception ex)
            {
                plugin.PlayniteApi.Dialogs.ShowErrorMessage($"Ошибка авторизации: {ex.Message}", "Self-Hosted Launcher");
            }
            return false;
        }

        private class TokenData
        {
            public string access_token { get; set; }
            public string token_type { get; set; }
        }
    }
}
