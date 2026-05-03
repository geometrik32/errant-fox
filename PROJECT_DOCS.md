# Errant Fox — Полная документация проекта

> **Назначение этого файла:** единая точка входа для AI-ассистента (и разработчика) для понимания структуры, архитектуры и всех файлов проекта. Читать в первую очередь.

---

## Оглавление

1. [Обзор проекта](#1-обзор-проекта)
2. [Стек технологий](#2-стек-технологий)
3. [Структура папок](#3-структура-папок)
4. [Архитектура взаимодействия](#4-архитектура-взаимодействия)
5. [Backend: полный разбор файлов и функций](#5-backend-полный-разбор-файлов-и-функций)
6. [Frontend: полный разбор файлов и функций](#6-frontend-полный-разбор-файлов-и-функций)
7. [База данных](#7-база-данных)
8. [REST API: все эндпоинты](#8-rest-api-все-эндпоинты)
9. [WebSocket API](#9-websocket-api)
10. [Инфраструктура (Docker / Nginx)](#10-инфраструктура-docker--nginx)
11. [Документация в docs/](#11-документация-в-docs)
12. [Архив (\_archive/)](#12-архив-_archive)

---

## 1. Обзор проекта

**Errant Fox** — веб-приложение для видеоанализа тренировочных спаррингов по HEMA (Historical European Martial Arts, длинный меч). Размещается на домашнем сервере TrueNAS SCALE в Docker-контейнерах.

**Ключевые функции:**
- Галерея видео с превью-кадрами и фильтрацией по бойцам/датам
- Видеоплеер с покадровым шагом, лупой, зумом
- Система разметки «сходов» (bouts): два бойца обмениваются ударами в пределах временного интервала
- Судейская панель: очки, техники, зоны поражения, результаты ударов
- Чат с комментариями, привязанными к таймкодам видео
- Статистика бойцов: графики частоты боёв, динамика результатов, прогресс по баллам, силуэты зон поражения
- Админ-панель: управление пользователями и списком техник

---

## 2. Стек технологий

| Слой | Технология | Назначение |
|---|---|---|
| **Frontend** | Svelte 5 + TypeScript + Vite | SPA, UI, видеоплеер, графики |
| **Backend** | Rust + Axum 0.8 + Tokio | REST API, бизнес-логика, WebSocket |
| **ORM** | Diesel 2.3 | Типизированные запросы к SQLite |
| **База данных** | SQLite (один файл) | Пользователи, видео-метаданные, буты, комментарии |
| **Хранение видео** | Seafile (отдельный HTTP-сервис) | Видеофайлы, стриминг |
| **Превью-кадры** | FFmpeg (запускается из бэкенда) | JPG-кадры для галереи |
| **Real-time** | WebSocket (broadcast) | Новые комментарии, обновления бутов, новые видео |
| **Деплой** | Docker Compose + Nginx + Traefik | Контейнеризация, проксирование, SSL |

---

## 3. Структура папок

```
Errant Fox/
├── PROJECT_DOCS.md          ← ЭТОТ ФАЙЛ — главная документация
├── deploy.md                ← Инструкция по деплою на TrueNAS
├── .gitignore
│
├── backend/                 ← Rust-сервер (Axum + SQLite)
│   ├── Cargo.toml           ← Зависимости Rust
│   ├── Cargo.lock
│   ├── .env.example         ← Шаблон переменных окружения
│   ├── migrations/          ← SQL-миграции Diesel
│   │   ├── 0001_initial/
│   │   ├── 0002_comment_reactions/
│   │   ├── 0003_comment_bout_search/
│   │   └── 0004_technique_description/
│   └── src/
│       ├── main.rs          ← Точка входа
│       ├── config.rs        ← Конфигурация из ENV
│       ├── errors.rs        ← Типы ошибок
│       ├── state.rs         ← Общее состояние приложения
│       ├── previews.rs      ← Генерация превью FFmpeg
│       ├── seafile.rs       ← HTTP-клиент Seafile API
│       ├── sync.rs          ← Фоновый опрос Seafile
│       ├── ws.rs            ← WebSocket хаб и обработчик
│       ├── db/
│       │   ├── mod.rs       ← Инициализация пула + миграции
│       │   ├── models.rs    ← Rust-структуры таблиц (Diesel)
│       │   └── schema.rs    ← Diesel schema (table! макросы)
│       ├── api/
│       │   ├── mod.rs       ← Axum router — все маршруты
│       │   ├── auth.rs      ← Авторизация + JWT
│       │   ├── bouts.rs     ← CRUD сходов
│       │   ├── comments.rs  ← CRUD комментариев + реакции + поиск
│       │   ├── techniques.rs← CRUD техник
│       │   ├── users.rs     ← Профили, бойцы, админ
│       │   └── videos.rs    ← CRUD видео, стриминг, превью
│       └── middleware/
│           ├── mod.rs
│           └── auth.rs      ← JWT-извлечение пользователя
│
├── frontend/                ← Svelte 5 SPA
│   ├── package.json
│   ├── svelte.config.js
│   ├── tsconfig.json
│   ├── index.html           ← Точка входа HTML
│   ├── public/
│   │   └── logo.png
│   └── src/
│       ├── main.ts          ← Mount Svelte-приложения
│       ├── App.svelte       ← Корневой компонент (роутинг)
│       ├── stores.ts        ← Глобальные Svelte stores
│       ├── lib/
│       │   ├── api/         ← HTTP-клиенты к бэкенду
│       │   │   ├── client.ts    ← Общий fetch-враппер
│       │   │   ├── auth.ts      ← Авторизация
│       │   │   ├── bouts.ts     ← Сходы
│       │   │   ├── comments.ts  ← Комментарии
│       │   │   ├── fighters.ts  ← Бойцы + админ-пользователи
│       │   │   ├── techniques.ts← Техники
│       │   │   ├── types.ts     ← TypeScript-типы + helpers
│       │   │   └── videos.ts    ← Видео
│       │   ├── player/      ← Компоненты видеоплеера
│       │   │   ├── VideoPlayer.svelte
│       │   │   ├── Chat.svelte
│       │   │   ├── JudgingPanel.svelte
│       │   │   ├── Timeline.svelte
│       │   │   ├── BoutCard.svelte
│       │   │   └── HitZonePicker.svelte
│       │   ├── gallery/     ← Компоненты галереи
│       │   │   ├── Sidebar.svelte
│       │   │   ├── VideoCard.svelte
│       │   │   └── VideoGrid.svelte
│       │   ├── stats/       ← Компоненты статистики
│       │   │   ├── BodySilhouette.svelte
│       │   │   ├── FighterSidebar.svelte
│       │   │   ├── FrequencyChart.svelte
│       │   │   ├── HistoryTable.svelte
│       │   │   ├── QuickStats.svelte
│       │   │   ├── ResultsChart.svelte
│       │   │   └── ScoreChart.svelte
│       │   ├── ui/          ← Общие UI-компоненты
│       │   │   ├── Header.svelte
│       │   │   ├── ProfileModal.svelte
│       │   │   └── SearchPanel.svelte
│       │   └── admin/       ← Админ-компоненты
│       │       ├── CreateUserModal.svelte
│       │       └── TechniquesModal.svelte
│       └── routes/          ← Страницы приложения
│           ├── Auth.svelte      ← Экран входа
│           ├── Gallery.svelte   ← Галерея видео
│           ├── Player.svelte    ← Видеоплеер
│           └── Stats.svelte     ← Статистика бойца
│
├── infra/                   ← Docker-инфраструктура
│   ├── docker-compose.yml
│   ├── Dockerfile.backend
│   ├── Dockerfile.frontend
│   └── nginx.conf
│
├── docs/                    ← Проектная документация (дополнительная)
│   ├── requirements.md
│   ├── architecture.md
│   ├── api.md
│   ├── database.md
│   └── implementation_plan.md
│
├── scripts/                 ← Пустая папка
│
└── _archive/                ← Архив старого кода (Clapshot)
    ├── backend_rust/        ← Старый Rust-бэкенд (gRPC, Diesel)
    └── frontend_noodl/      ← Старый Noodl-фронтенд
```

---

## 4. Архитектура взаимодействия

```
[Браузер]  ←──REST (JSON)──→  [Backend: Rust/Axum :8080]  ←──HTTP──→  [Seafile]
    │                              │                │
    │                              ├── SQLite        ├── FFmpeg (превью)
    │                              ├── WebSocket     └── Файловая система
    │                              └── Broadcast
    │
    └── Стриминг видео напрямую с Seafile (байты не проходят через бэкенд)
```

- **REST** — все CRUD-операции (видео, буты, комментарии, пользователи, техники)
- **WebSocket** — только live-события (новый комментарий, обновление/удаление бута, новое видео из Seafile)
- **Seafile** — отдельный сервис; бэкенд ходит к нему за списком файлов + download-ссылками; браузер стримит видео напрямую
- **FFmpeg** — запускается бэкендом для генерации превью-кадров при первом запросе

---

## 5. Backend: полный разбор файлов и функций

### 5.1. [`backend/Cargo.toml`](backend/Cargo.toml)

Конфигурация Rust-пакета `errant_fox`. Ключевые зависимости:
- **axum 0.8** — веб-фреймворк (multipart, ws)
- **tokio** — асинхронный рантайм
- **diesel 2.3** — ORM (sqlite, r2d2 pool, chrono)
- **jsonwebtoken** — JWT-токены
- **bcrypt** — хеширование паролей
- **reqwest** — HTTP-клиент к Seafile
- **uuid** — генерация UUID v4

---

### 5.2. [`backend/src/main.rs`](backend/src/main.rs)

**Точка входа** приложения. Единственная публичная функция:

#### `async fn main()`
1. Инициализирует `tracing_subscriber` для логирования
2. Загружает [`config::Config`](backend/src/config.rs) из переменных окружения
3. Создаёт [`seafile::SeafileClient`](backend/src/seafile.rs)
4. Инициализирует пул БД через [`db::init_pool()`](backend/src/db/mod.rs)
5. Создаёт broadcast-канал `tokio::sync::broadcast::channel::<WsEvent>(256)` для WebSocket
6. Запускает фоновый [`sync::run_sync()`](backend/src/sync.rs) через `tokio::spawn`
7. Собирает [`state::AppState`](backend/src/state.rs)
8. Настраивает CORS (`tower_http::cors::CorsLayer`)
9. Строит роутер через [`api::router()`](backend/src/api/mod.rs)
10. Запускает `axum::serve` на `0.0.0.0:{SERVER_PORT}`

---

### 5.3. [`backend/src/config.rs`](backend/src/config.rs)

#### `struct Config`
Поля (все из ENV):
| Поле | Тип | ENV-переменная |
|---|---|---|
| `database_url` | `String` | `DATABASE_URL` |
| `jwt_secret` | `String` | `JWT_SECRET` |
| `seafile_url` | `String` | `SEAFILE_URL` |
| `seafile_token` | `String` | `SEAFILE_TOKEN` |
| `previews_dir` | `String` | `PREVIEWS_DIR` |
| `avatars_dir` | `String` | `AVATARS_DIR` |
| `server_port` | `u16` | `SERVER_PORT` |
| `frontend_origin` | `String` | `FRONTEND_ORIGIN` |

#### `fn from_env() -> Config`
Загружает `.env` через `dotenvy`, читает все переменные.

#### `fn required(key: &str) -> String`
Помошник — получает ENV-переменную или паникует.

---

### 5.4. [`backend/src/state.rs`](backend/src/state.rs)

#### `struct AppState`
Общее состояние приложения, передаваемое во все хендлеры через `axum::extract::State`:
| Поле | Тип | Назначение |
|---|---|---|
| `db` | `DbPool` | Пул соединений SQLite |
| `jwt_secret` | `String` | Секрет для подписи JWT |
| `avatars_dir` | `String` | Путь к папке с аватарами |
| `previews_dir` | `String` | Путь к папке с превью-кадрами |
| `seafile` | `Arc<SeafileClient>` | HTTP-клиент Seafile |
| `ws_hub` | `WsHub` | Broadcast-отправитель WebSocket-событий |

---

### 5.5. [`backend/src/errors.rs`](backend/src/errors.rs)

#### `enum AppError`
Централизованный тип ошибок с вариантами:
- `Unauthorized(String)` → HTTP 401
- `Forbidden` → HTTP 403
- `NotFound` → HTTP 404
- `BadRequest(String)` → HTTP 400
- `Internal(String)` → HTTP 500 (сообщение логируется, клиенту отдаётся "Internal server error")

#### `impl IntoResponse for AppError`
Автоматически конвертирует ошибку в HTTP-ответ с JSON-телом `{"error": "..."}`.

---

### 5.6. [`backend/src/previews.rs`](backend/src/previews.rs)

#### `async fn get_duration(url: &str) -> f64`
Запускает `ffprobe` для получения длительности видео по URL. При ошибке возвращает 60.0 по умолчанию.

#### `pub async fn generate_previews(video_id: &str, download_url: &str, previews_dir: &Path, db: &DbPool) -> Result<(), AppError>`
1. Создаёт папку `{previews_dir}/{video_id}/`
2. Определяет длительность видео через `get_duration()`
3. Запускает `ffmpeg` — делает seek в середину видео, извлекает 1 кадр (JPG, ширина 480px) в файл `0.jpg` (флаг `-start_number 0`)
4. Обновляет `videos.preview_count = 1` в БД

---

### 5.7. [`backend/src/seafile.rs`](backend/src/seafile.rs)

#### `struct SeafileClient`
HTTP-клиент к Seafile REST API (v2.1 via-repo-token). Хранит `url`, `token` и `reqwest::Client`.

#### `fn new(url: String, token: String) -> Arc<Self>`
Создаёт новый экземпляр, обёрнутый в `Arc`.

#### `fn auth_header(&self) -> String`
Возвращает `"Bearer {token}"` для заголовка Authorization.

#### `pub async fn list_folders(&self) -> Result<Vec<FolderInfo>>`
`GET /api/v2.1/via-repo-token/dir/?path=/` — список папок в корне репозитория. Фильтрует только `"type": "dir"`.

#### `pub async fn list_files(&self, folder: &str) -> Result<Vec<FileInfo>>`
`GET /api/v2.1/via-repo-token/dir/?path=/folder` — список файлов в папке. Фильтрует только `"type": "file"`.

#### `pub async fn get_download_url(&self, path: &str) -> Result<String>`
`GET /api/v2.1/via-repo-token/download-link/?path=/path` — получает временную ссылку на скачивание.

#### `pub async fn fetch_range(&self, path: &str, range: Option<&str>) -> Result<reqwest::Response>`
Проксирует запрос к файлу через download URL (без стриминга байтов через бэкенд). Поддерживает HTTP Range.

---

### 5.8. [`backend/src/sync.rs`](backend/src/sync.rs)

#### `pub async fn run_sync(seafile, db, ws_tx)`
Бесконечный цикл с интервалом 60 секунд. Вызывает `sync_once()`, логирует ошибки.

#### `async fn sync_once(seafile, db, ws_tx, date_re) -> Result<()>`
1. Получает список папок из Seafile
2. Для каждой папки пытается извлечь дату через regex `(\d{4}[.\-]\d{2}[.\-]\d{2})`
3. Парсит дату (форматы `YYYY.MM.DD` или `YYYY-MM-DD`)
4. Получает список файлов в папке
5. Для каждого файла проверяет, нет ли уже записи в `videos` с таким `seafile_path`
6. Если нет — создаёт новую запись с UUID, путём, датой и отправляет `WsEvent::NewVideo` в WebSocket-хаб

---

### 5.9. [`backend/src/ws.rs`](backend/src/ws.rs)

#### DTO-типы
- **`WsCommentAuthor`** — `id, display_name, avatar_url, color`
- **`WsComment`** — `id, video_id, author, timestamp_ms, text, reply_to_id, created_at, edited_at?, bout_id?`
- **`WsBout`** — полные данные схода (все поля `bouts` таблицы + `video_id`)

#### `enum WsEvent`
События WebSocket, сериализуются с тегом `"type"`:
- `NewComment(WsComment)` — новый комментарий
- `UpdateBout(WsBout)` — создан/изменён сход
- `NewVideo { id, date, preview_url }` — обнаружено новое видео

#### `fn video_id(&self) -> Option<&str>`
Возвращает `video_id` для фильтрации: `NewVideo` → `None` (broadcast всем), остальные → `Some` (только зрителям этого видео).

#### `pub type WsHub = broadcast::Sender<WsEvent>`
Тип-алиас для WebSocket-хаба.

#### `pub async fn ws_handler(ws, state) -> impl IntoResponse`
Axum-хендлер для апгрейда HTTP → WebSocket на `/ws`.

#### `async fn handle_socket(socket, state)`
1. Ждёт первое сообщение — JSON `{"token": "..."}` → верифицирует JWT
2. Подписывается на `state.ws_hub`
3. В цикле `tokio::select!`: принимает сообщения от клиента (поле `"watching": "video_id"` задаёт фильтр) ИЛИ получает события из хаба и отправляет клиенту (с фильтрацией по `video_id`)

---

### 5.10. [`backend/src/db/mod.rs`](backend/src/db/mod.rs)

#### `pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>`
Тип пула соединений.

#### `pub fn init_pool(database_url: &str) -> DbPool`
1. Устанавливает прямое соединение к SQLite
2. Запускает `diesel_migrations::MigrationHarness::run_pending_migrations` (миграции из `backend/migrations/`)
3. Создаёт `r2d2::Pool`

---

### 5.11. [`backend/src/db/models.rs`](backend/src/db/models.rs)

Rust-структуры, зеркалирующие таблицы SQLite (Diesel `Queryable` + `Insertable`):

#### `User` / `NewUser` — таблица `users`
| Поле | Тип |
|---|---|
| `id` | `String` (UUID) |
| `username` | `String` |
| `display_name` | `String` |
| `password_hash` | `String` |
| `is_admin` | `bool` |
| `avatar_path` | `Option<String>` |
| `color` | `Option<String>` |
| `created_at` | `NaiveDateTime` |

#### `Video` / `NewVideo` — таблица `videos`
| Поле | Тип |
|---|---|
| `id` | `String` (UUID) |
| `seafile_path` | `String` |
| `fighter_a_id` | `Option<String>` |
| `fighter_b_id` | `Option<String>` |
| `date` | `NaiveDate` |
| `duration_ms` | `Option<i32>` |
| `preview_count` | `i32` |
| `created_at` | `NaiveDateTime` |

#### `Technique` / `NewTechnique` — таблица `techniques`
| Поле | Тип |
|---|---|
| `id` | `i32` |
| `name` | `String` |
| `description` | `Option<String>` |

#### `Bout` / `NewBout` — таблица `bouts`
| Поле | Тип |
|---|---|
| `id` | `i32` |
| `video_id` | `String` |
| `order_index` | `i32` |
| `time_start_ms` | `i32` |
| `time_end_ms` | `i32` |
| `score_a` | `i32` |
| `score_b` | `i32` |
| `technique_a_id` | `Option<i32>` |
| `technique_b_id` | `Option<i32>` |
| `hit_zone_a` | `Option<String>` |
| `hit_zone_b` | `Option<String>` |
| `result_a` | `Option<String>` |
| `result_b` | `Option<String>` |

#### `Comment` / `NewComment` — таблица `comments`
| Поле | Тип |
|---|---|
| `id` | `i32` |
| `video_id` | `String` |
| `author_id` | `String` |
| `timestamp_ms` | `i32` |
| `text` | `String` |
| `reply_to_id` | `Option<i32>` |
| `created_at` | `NaiveDateTime` |
| `edited_at` | `Option<NaiveDateTime>` |
| `bout_id` | `Option<i32>` |

#### `CommentReaction` — таблица `comment_reactions`
| Поле | Тип |
|---|---|
| `comment_id` | `i32` |
| `user_id` | `String` |
| `kind` | `String` (`"like"` / `"dislike"`) |

---

### 5.12. [`backend/src/db/schema.rs`](backend/src/db/schema.rs)

Diesel `table!` макросы для всех 6 таблиц + `joinable!` и `allow_tables_to_appear_in_same_query!` макросы. Генерируется Diesel CLI, но в проекте хранится вручную.

---

### 5.13. [`backend/src/api/mod.rs`](backend/src/api/mod.rs)

#### `pub fn router(state: AppState) -> Router`
Строит полный Axum-роутер со всеми маршрутами:

| Метод | Путь | Хендлер |
|---|---|---|
| POST | `/api/auth/login` | [`auth::login`](backend/src/api/auth.rs) |
| GET | `/api/users/me` | [`auth::get_me`](backend/src/api/auth.rs) |
| PATCH | `/api/users/me` | [`users::patch_me`](backend/src/api/users.rs) |
| POST | `/api/users/me/avatar` | [`users::upload_avatar`](backend/src/api/users.rs) |
| GET | `/api/users/{id}/avatar` | [`users::get_avatar`](backend/src/api/users.rs) |
| GET | `/api/fighters` | [`users::list_fighters`](backend/src/api/users.rs) |
| GET | `/api/fighters/{id}/bouts` | [`users::fighter_bouts`](backend/src/api/users.rs) |
| POST | `/api/admin/users` | [`users::create_user`](backend/src/api/users.rs) |
| PATCH | `/api/admin/users/{id}` | [`users::patch_admin_user`](backend/src/api/users.rs) |
| DELETE | `/api/admin/users/{id}` | [`users::delete_user`](backend/src/api/users.rs) |
| POST | `/api/admin/users/{id}/avatar` | [`users::upload_avatar_for`](backend/src/api/users.rs) |
| GET | `/api/techniques` | [`techniques::list_techniques`](backend/src/api/techniques.rs) |
| POST | `/api/admin/techniques` | [`techniques::create_technique`](backend/src/api/techniques.rs) |
| PATCH | `/api/admin/techniques/{id}` | [`techniques::patch_technique`](backend/src/api/techniques.rs) |
| DELETE | `/api/admin/techniques/{id}` | [`techniques::delete_technique`](backend/src/api/techniques.rs) |
| GET | `/api/videos` | [`videos::list_videos`](backend/src/api/videos.rs) |
| GET | `/api/videos/{id}` | [`videos::get_video`](backend/src/api/videos.rs) |
| PATCH | `/api/videos/{id}` | [`videos::patch_video`](backend/src/api/videos.rs) |
| GET | `/api/videos/{id}/stream` | [`videos::stream_video`](backend/src/api/videos.rs) |
| GET | `/api/videos/{id}/previews/{frame}` | [`videos::get_preview_frame`](backend/src/api/videos.rs) |
| POST | `/api/bouts` | [`bouts::post_bout`](backend/src/api/bouts.rs) |
| PATCH | `/api/bouts/{id}` | [`bouts::patch_bout`](backend/src/api/bouts.rs) |
| DELETE | `/api/bouts/{id}` | [`bouts::delete_bout`](backend/src/api/bouts.rs) |
| POST | `/api/comments` | [`comments::post_comment`](backend/src/api/comments.rs) |
| GET | `/api/comments/search` | [`comments::search_comments`](backend/src/api/comments.rs) |
| PATCH | `/api/comments/{id}` | [`comments::patch_comment`](backend/src/api/comments.rs) |
| DELETE | `/api/comments/{id}` | [`comments::delete_comment`](backend/src/api/comments.rs) |
| POST | `/api/comments/{id}/react` | [`comments::react_comment`](backend/src/api/comments.rs) |
| DELETE | `/api/comments/{id}/react` | [`comments::delete_react`](backend/src/api/comments.rs) |
| GET | `/ws` | [`crate::ws::ws_handler`](backend/src/ws.rs) |

Все маршруты (кроме `/api/auth/login`) требуют JWT-токен через [`middleware::auth::CurrentUser`](backend/src/middleware/auth.rs). Админ-маршруты дополнительно проверяют `user.is_admin`.

---

### 5.14. [`backend/src/api/auth.rs`](backend/src/api/auth.rs)

#### `fn generate_color(user_id: &str) -> String`
Детерминированно выбирает HEX-цвет из палитры 12 цветов на основе суммы байтов `user_id`.

#### `struct Claims` — `{ sub: String, exp: usize }`
JWT-claims: subject = user_id, expiration = UNIX timestamp.

#### `fn make_token(user_id: &str, secret: &str) -> Result<String, AppError>`
Создаёт JWT с алгоритмом HS256, сроком 7 дней.

#### `fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError>`
Декодирует и верифицирует JWT.

#### DTO-типы
- **`UserDto`** — публичные данные пользователя (`id, username, display_name, is_admin, avatar_url, color`)
- **`UserMeDto`** — данные текущего пользователя (отличается тем, что `color` — `Option<String>`, показывает настоящий цвет)
- **`LoginRequest`** — `{ username, password }`
- **`LoginResponse`** — `{ token, user: UserDto }`
- **`PatchMeRequest`** — `{ display_name?, password? }`

#### `fn to_user_dto(u: &User) -> UserDto`
Конвертирует модель БД в DTO (генерирует цвет если отсутствует).

#### `fn to_me_dto(u: &User) -> UserMeDto`
Конвертирует модель БД в MeDTO (сохраняет реальный цвет из БД).

#### `pub async fn login(state, body) -> Result<Json<LoginResponse>, AppError>`
1. Ищет пользователя по `username`
2. Проверяет пароль через `bcrypt::verify`
3. Если `color` не назначен — генерирует и сохраняет в БД
4. Создаёт JWT-токен
5. Возвращает токен + данные пользователя

#### `pub async fn get_me(state, user) -> Result<Json<UserMeDto>, AppError>`
Возвращает данные текущего пользователя. Если цвет не назначен — генерирует и сохраняет.

#### `pub async fn patch_me(state, user, body) -> Result<Json<UserMeDto>, AppError>`
Обновляет `display_name` и/или `password` (с хешированием через bcrypt) текущего пользователя.

---

### 5.15. [`backend/src/api/bouts.rs`](backend/src/api/bouts.rs)

#### DTO / Request
- **`BoutResponse`** — все поля схода для ответа API
- **`CreateBoutRequest`** — `{ video_id, time_start_ms, time_end_ms }`
- **`PatchBoutRequest`** — все поля опциональны; nullable-поля используют `deser_opt_nullable` для различения "не передано" / "передан null" / "передано значение"

#### `fn to_response(b: &Bout) -> BoutResponse`
Конвертирует модель БД в ответ API.

#### `fn to_ws_bout(b: &Bout) -> WsBout`
Конвертирует модель БД в WebSocket-событие.

#### `fn deser_opt_nullable<'de, T, D>(deserializer) -> Result<Option<Option<T>>, D::Error>`
Кастомный десериализатор: `None` = поле отсутствует, `Some(None)` = явный null, `Some(Some(v))` = значение.

#### `pub async fn post_bout(state, user, body) -> Result<(StatusCode, Json<BoutResponse>), AppError>`
1. Проверяет существование `video_id`
2. Вычисляет `order_index = MAX(order_index) + 1` для этого видео
3. Вставляет новый сход с дефолтными значениями
4. Отправляет `WsEvent::UpdateBout` в WebSocket-хаб
5. Возвращает `201 Created`

#### `pub async fn patch_bout(state, user, Path(id), body) -> Result<Json<BoutResponse>, AppError>`
Обновляет указанные поля схода. Отправляет `WsEvent::UpdateBout`.

#### `pub async fn delete_bout(state, user, Path(id)) -> Result<Json<Value>, AppError>`
Удаляет сход по ID. Возвращает `{"ok": true}`.

---

### 5.16. [`backend/src/api/comments.rs`](backend/src/api/comments.rs)

#### DTO / Request
- **`CommentAuthorResponse`** — `{ id, display_name, avatar_url, color? }`
- **`CommentResponse`** — полный комментарий + `likes, dislikes, my_reaction`
- **`CreateCommentRequest`** — `{ video_id, timestamp_ms, text, reply_to_id? }`
- **`PatchCommentRequest`** — `{ text }`
- **`ReactRequest`** — `{ kind: "like" | "dislike" }`
- **`SearchQuery`** — `{ q: String }`
- **`SearchResult`** — результат поиска с данными видео и бойцов

#### `fn to_response(c, author, likes, dislikes, my_reaction) -> CommentResponse`
#### `fn load_reactions(comment_id, user_id, conn) -> Result<(i32, i32, Option<String>), AppError>`
Считает лайки/дизлайки и свою реакцию для комментария.

#### `fn to_ws_comment(c, author) -> WsComment`
Конвертирует в WebSocket-событие.

#### `pub async fn post_comment(state, user, body) -> Result<(StatusCode, Json<CommentResponse>), AppError>`
1. Проверяет существование видео
2. Авто-определяет `bout_id` — какой сход содержит этот таймкод (SQL: `time_start_ms <= ts AND time_end_ms >= ts`)
3. Вставляет комментарий
4. Отправляет `WsEvent::NewComment`
5. Возвращает `201 Created`

#### `pub async fn patch_comment(state, user, Path(id), body) -> Result<Json<CommentResponse>, AppError>`
Редактирует текст (только свой комментарий). Устанавливает `edited_at`.

#### `pub async fn delete_comment(state, user, Path(id)) -> Result<Json<Value>, AppError>`
Каскадное удаление: удаляет реакции на ответы → удаляет ответы → удаляет реакции на комментарий → удаляет комментарий. Свой комментарий или Admin.

#### `pub async fn react_comment(state, user, Path(id), body) -> Result<Json<Value>, AppError>`
`INSERT OR REPLACE` (через `replace_into`) реакцию `like`/`dislike`.

#### `pub async fn delete_react(state, user, Path(id)) -> Result<Json<Value>, AppError>`
Удаляет свою реакцию с комментария.

#### `pub async fn search_comments(state, user, Query(params)) -> Result<Json<Vec<SearchResult>>, AppError>`
Полнотекстовый поиск по `comments.text` через SQL `LIKE`. JOIN с `users`, `videos`, опционально `bouts`. Лимит 50 результатов.

---

### 5.17. [`backend/src/api/techniques.rs`](backend/src/api/techniques.rs)

#### DTO / Request
- **`CreateTechniqueRequest`** — `{ name, description? }`
- **`PatchTechniqueRequest`** — `{ name?, description? }`

#### `pub async fn list_techniques(state, user) -> Result<Json<Vec<Technique>>, AppError>`
Возвращает все техники из БД.

#### `pub async fn create_technique(state, user, body) -> Result<(StatusCode, Json<Technique>), AppError>`
Создаёт новую технику (Admin only). `201 Created`.

#### `pub async fn patch_technique(state, user, Path(id), body) -> Result<Json<Technique>, AppError>`
Обновляет имя/описание техники (Admin only).

#### `pub async fn delete_technique(state, user, Path(id)) -> Result<Json<Value>, AppError>`
1. Обнуляет `technique_a_id` и `technique_b_id` во всех бутах, ссылающихся на эту технику
2. Удаляет технику
3. Admin only

---

### 5.18. [`backend/src/api/users.rs`](backend/src/api/users.rs)

#### DTO / Request
- **`FighterDto`** — `{ id, username, display_name, avatar_url, color?, is_admin }`
- **`FighterBoutDto`** — полный бой с позиции бойца (`my_score`, `my_technique_*`, `opponent_*`)
- **`PatchMeRequest`** — `{ username?, display_name?, password?, color? }`
- **`CreateUserRequest`** — `{ username, display_name, password, is_admin, color? }`
- **`PatchAdminUserRequest`** — `{ display_name?, password?, color?, is_admin? }`

#### `pub async fn list_fighters(state, user) -> Result<Json<Vec<FighterDto>>, AppError>`
Возвращает всех пользователей как список бойцов.

#### `pub async fn fighter_bouts(state, user, Path(fighter_id)) -> Result<Json<Vec<FighterBoutDto>>, AppError>`
Сложный запрос для страницы статистики:
1. Находит все видео с участием бойца
2. Загружает все буты этих видео
3. Загружает техники и оппонентов
4. Для каждого бута определяет, является ли боец A или B, и строит DTO с позиции бойца

#### `pub async fn patch_me(state, user, body) -> Result<Json<UserMeDto>, AppError>`
Обновляет профиль: `username`, `display_name`, `password`, `color`.

#### `pub async fn upload_avatar(state, user, multipart) -> Result<Json<Value>, AppError>`
Принимает multipart-форму с полем `avatar` (макс 2MB), сохраняет как `{avatars_dir}/{user_id}.jpg`.

#### `pub async fn get_avatar(state, Path(user_id)) -> Result<Response, AppError>`
Отдаёт JPEG-файл аватара с диска.

#### `pub async fn create_user(state, user, body) -> Result<(StatusCode, Json<UserMeDto>), AppError>`
Создаёт пользователя (Admin only). Генерирует UUID, хеширует пароль bcrypt. `201 Created`.

#### `pub async fn patch_admin_user(state, user, Path(id), body) -> Result<Json<UserMeDto>, AppError>`
Редактирование пользователя админом (Admin only).

#### `pub async fn upload_avatar_for(state, user, Path(id), multipart) -> Result<Json<Value>, AppError>`
Загрузка аватара для указанного пользователя (Admin only).

#### `pub async fn delete_user(state, user, Path(id)) -> Result<Json<Value>, AppError>`
Удаление пользователя (Admin only). Нельзя удалить самого себя.

---

### 5.19. [`backend/src/api/videos.rs`](backend/src/api/videos.rs)

#### DTO / Request
- **`VideoFighterDto`** — `{ id, display_name, avatar_url, color? }`
- **`VideoListDto`** — карточка для галереи: `id, date, fighter_a?, fighter_b?, total_score_a?, total_score_b?, is_tagged, preview_url, preview_count`
- **`VideoFullDto`** — полные данные видео: `+ stream_url, duration_ms, bouts[], comments[]`
- **`BoutDto`** — сход внутри VideoFull
- **`CommentDto`** — комментарий внутри VideoFull (с `likes, dislikes, my_reaction`)
- **`VideoListQuery`** — query-параметры: `fighter_id?, date_from?, date_to?`
- **`PatchVideoRequest`** — `{ fighter_a_id?, fighter_b_id? }`

#### Helper-функции
- **`fighter_dto(u) -> VideoFighterDto`**
- **`bout_dto(b) -> BoutDto`**
- **`build_video_full(video, bouts, comments, users_map, reactions_map, stream_url) -> VideoFullDto`** — собирает полный DTO
- **`load_users_for_video(video, comments, conn) -> HashMap<String, User>`** — загружает пользователей (бойцы + авторы комментариев) одним запросом
- **`build_reactions_map(reactions, current_user_id) -> HashMap<i32, (likes, dislikes, my_reaction)>`** — аггрегирует реакции по comment_id

#### `pub async fn list_videos(state, user, Query(params)) -> Result<Json<Vec<VideoListDto>>, AppError>`
Фильтрация по `fighter_id`, `date_from`, `date_to`. Сортировка по дате (новые сначала). Вычисляет `total_score_a/b` из бутов. Возвращает `is_tagged = fighter_a_id IS NOT NULL AND fighter_b_id IS NOT NULL`.

#### `pub async fn get_video(state, user, Path(video_id)) -> Result<Json<VideoFullDto>, AppError>`
Загружает видео + все буты + все комментарии + реакции + пользователей. Возвращает полный DTO с `stream_url`.

#### `pub async fn patch_video(state, user, Path(video_id), body) -> Result<Json<VideoFullDto>, AppError>`
Назначает/меняет бойцов на видео.

#### `pub async fn get_preview_frame(state, Path((video_id, frame))) -> Result<Response, AppError>`
1. Если `preview_count == 0` — запускает `generate_previews()` в фоне (spawn), возвращает `202 Accepted`
2. Иначе читает `{previews_dir}/{video_id}/{frame}.jpg` и отдаёт как `image/jpeg`

#### `pub async fn stream_video(state, Path(video_id), req_headers) -> Result<Response, AppError>`
Проксирует видео с Seafile:
1. Находит `seafile_path` в БД
2. Проксирует Range-заголовок в [`SeafileClient::fetch_range()`](backend/src/seafile.rs)
3. Проксирует ответ (status, content-type, content-length, content-range) обратно клиенту как `Body::from_stream`

---

### 5.20. [`backend/src/middleware/auth.rs`](backend/src/middleware/auth.rs)

#### `pub struct CurrentUser(pub User)`
Newtype-обёртка для извлечения аутентифицированного пользователя в хендлерах.

#### `impl FromRequestParts<S> for CurrentUser`
Axum-экстрактор:
1. Читает заголовок `Authorization: Bearer <token>`
2. Верифицирует JWT через [`auth::verify_token()`](backend/src/api/auth.rs)
3. Загружает пользователя из БД по `claims.sub`
4. Возвращает `CurrentUser(user)` или `AppError::Unauthorized`

---

## 6. Frontend: полный разбор файлов и функций

### 6.1. Точки входа

#### [`frontend/src/main.ts`](frontend/src/main.ts)
```typescript
mount(App, { target: document.body })
```
Монтирует корневой Svelte-компонент в `<body>`.

#### [`frontend/src/App.svelte`](frontend/src/App.svelte)
Корневой компонент. Реализует:
- Hash-based роутинг (`window.location.hash`): `#/gallery`, `#/stats`, `#/player/{videoId}?t={ms}`
- Состояния: `initialized`, `routeName`, `playerId`, `initialTimeMs`
- Условный рендеринг: если нет токена → [`Auth`](frontend/src/routes/Auth.svelte), иначе → [`Header`](frontend/src/lib/ui/Header.svelte) + страница
- Вызывает [`initStores()`](frontend/src/stores.ts) при наличии токена

---

### 6.2. Глобальное состояние

#### [`frontend/src/stores.ts`](frontend/src/stores.ts)

**Svelte writable stores:**
- **`token: Writable<string | null>`** — JWT-токен (читается/пишется в `localStorage` под ключом `ef_token`)
- **`currentUser: Writable<User | null>`** — данные текущего пользователя
- **`techniques: Writable<Technique[]>`** — кеш списка техник
- **`fighters: Writable<Fighter[]>`** — кеш списка бойцов

**`token.subscribe()`** — авто-сохранение/удаление токена в `localStorage`.

#### `async function initStores(): Promise<void>`
Параллельно загружает бойцов, техники и профиль пользователя через API, заполняет stores.

---

### 6.3. API-клиенты

#### [`frontend/src/lib/api/client.ts`](frontend/src/lib/api/client.ts)

##### `async function apiFetch<T>(path: string, options?: RequestInit): Promise<T>`
Универсальная функция для запросов к бэкенду:
1. Добавляет заголовки `Content-Type: application/json` и `Authorization: Bearer {token}`
2. Делает `fetch` к `/api{path}`
3. При 401 — удаляет токен, редиректит на `/auth`
4. При других ошибках — выбрасывает `Error` с текстом
5. Парсит JSON-ответ

#### [`frontend/src/lib/api/auth.ts`](frontend/src/lib/api/auth.ts)

- **`login(username, password): Promise<LoginResponse>`** — `POST /api/auth/login`
- **`getMe(): Promise<User>`** — `GET /api/users/me`
- **`patchMe(data): Promise<User>`** — `PATCH /api/users/me`
- **`uploadMyAvatar(file): Promise<{avatar_url}>`** — `POST /api/users/me/avatar` (multipart)

#### [`frontend/src/lib/api/bouts.ts`](frontend/src/lib/api/bouts.ts)

- **`createBout(data: CreateBoutData): Promise<Bout>`** — `POST /api/bouts`
- **`updateBout(id, data: UpdateBoutData): Promise<Bout>`** — `PATCH /api/bouts/{id}`
- **`deleteBout(id): Promise<{ok}>`** — `DELETE /api/bouts/{id}`

Типы: `CreateBoutData` (`video_id, time_start_ms, time_end_ms`), `UpdateBoutData` (все поля опциональны, включая nullable).

#### [`frontend/src/lib/api/comments.ts`](frontend/src/lib/api/comments.ts)

- **`createComment(data: CreateCommentData): Promise<Comment>`** — `POST /api/comments`
- **`updateComment(id, text): Promise<Comment>`** — `PATCH /api/comments/{id}`
- **`deleteComment(id): Promise<{ok}>`** — `DELETE /api/comments/{id}`
- **`reactComment(id, kind): Promise<void>`** — `POST /api/comments/{id}/react`
- **`deleteReact(id): Promise<void>`** — `DELETE /api/comments/{id}/react`
- **`searchComments(q): Promise<SearchResult[]>`** — `GET /api/comments/search?q=...`

#### [`frontend/src/lib/api/fighters.ts`](frontend/src/lib/api/fighters.ts)

- **`getFighters(): Promise<Fighter[]>`** — `GET /api/fighters`
- **`getFighterBouts(id): Promise<FighterBout[]>`** — `GET /api/fighters/{id}/bouts`
- **`createUser(data): Promise<User>`** — `POST /api/admin/users` (Admin)
- **`patchUser(id, data): Promise<User>`** — `PATCH /api/admin/users/{id}` (Admin)
- **`deleteUser(id): Promise<void>`** — `DELETE /api/admin/users/{id}` (Admin)
- **`uploadUserAvatar(userId, file): Promise<{avatar_url}>`** — `POST /api/admin/users/{id}/avatar` (Admin, multipart)

#### [`frontend/src/lib/api/techniques.ts`](frontend/src/lib/api/techniques.ts)

- **`getTechniques(): Promise<Technique[]>`** — `GET /api/techniques`
- **`createTechnique(name, description?): Promise<Technique>`** — `POST /api/admin/techniques`
- **`patchTechnique(id, data): Promise<Technique>`** — `PATCH /api/admin/techniques/{id}`
- **`renameTechnique(id, name): Promise<Technique>`** — обёртка над patchTechnique
- **`deleteTechnique(id): Promise<{ok}>`** — `DELETE /api/admin/techniques/{id}`

#### [`frontend/src/lib/api/videos.ts`](frontend/src/lib/api/videos.ts)

- **`getVideos(filters?): Promise<Video[]>`** — `GET /api/videos?fighter_id=&date_from=&date_to=`
- **`getVideo(id): Promise<VideoFull>`** — `GET /api/videos/{id}`
- **`patchVideo(id, data): Promise<VideoFull>`** — `PATCH /api/videos/{id}`
- **`getStreamUrl(id): Promise<{stream_url}>`** — `GET /api/videos/{id}/stream`

#### [`frontend/src/lib/api/types.ts`](frontend/src/lib/api/types.ts)

TypeScript-интерфейсы, зеркалирующие ответы API:
- **`User`** — текущий пользователь
- **`Fighter`** — боец в списке
- **`FighterBout`** — бой для страницы статистики (с позиции бойца)
- **`VideoFighter`** — боец в контексте видео
- **`Video`** — карточка галереи
- **`VideoFull`** — полные данные видео (+ буты, комментарии)
- **`Bout`** — сход
- **`Comment`** — комментарий (+ `likes, dislikes, my_reaction`)
- **`Technique`** — техника
- **`SearchResult`** — результат поиска комментариев

Utility-функции:
- **`resolveColor(id, color?): string`** — возвращает HEX-цвет из БД или генерирует HSL по хешу ID
- **`buildVideoLabels(bouts, fighterName): Map<string, string>`** — строит человекочитаемые метки видео в формате `АБ_26.04.23_01`

---

### 6.4. Страницы (routes)

#### [`frontend/src/routes/Auth.svelte`](frontend/src/routes/Auth.svelte)
Страница входа. Форма с полями Username/Password, кнопка Sign In. Вызывает [`login()`](frontend/src/lib/api/auth.ts), сохраняет токен и пользователя в stores.

#### [`frontend/src/routes/Gallery.svelte`](frontend/src/routes/Gallery.svelte)
Галерея видео. Содержит:
- [`Sidebar`](frontend/src/lib/gallery/Sidebar.svelte) — фильтры по бойцам и диапазону дат
- [`VideoGrid`](frontend/src/lib/gallery/VideoGrid.svelte) — сетка [`VideoCard`](frontend/src/lib/gallery/VideoCard.svelte)
- Логика загрузки видео через [`getVideos()`](frontend/src/lib/api/videos.ts)
- Модальное окно для назначения бойцов на неразмеченное видео

#### [`frontend/src/routes/Player.svelte`](frontend/src/routes/Player.svelte)
Страница видеоплеера. Оркестрирует:
- [`VideoPlayer`](frontend/src/lib/player/VideoPlayer.svelte) — центральный плеер
- [`JudgingPanel`](frontend/src/lib/player/JudgingPanel.svelte) — левая панель разметки
- [`Chat`](frontend/src/lib/player/Chat.svelte) — правая панель чата
- [`Timeline`](frontend/src/lib/player/Timeline.svelte) — нижний таймлайн
- Управление состояниями: `currentTime`, `duration`, `playing`, `looping`, `speed`, `volume`, `fps`, `highlightedCommentId`
- Кнопки показа/скрытия панелей (JudgingPanel, Chat)
- Клавиатурные хоткеи: `Space` (play/pause), `F` (fullscreen), `X`/`Z` (frame-step), `←`/`→` (±2с), `A` (0.2×), `S` (2×), `C` (triggerMark), `G` (toggle panels)
- Клик по маркеру комментария на таймлайне → скролл и подсветка сообщения в чате

#### [`frontend/src/routes/Stats.svelte`](frontend/src/routes/Stats.svelte)
Страница статистики бойца. Содержит:
- Inline hero-карточка бойца с dropdown выбора бойца (FighterSidebar не используется)
- [`QuickStats`](frontend/src/lib/stats/QuickStats.svelte) — блоки быстрой статистики
- [`ScoreChart`](frontend/src/lib/stats/ScoreChart.svelte) — прогресс по баллам
- [`RecentOpponents`](frontend/src/lib/stats/RecentOpponents.svelte) — список оппонентов с win/loss
- [`FrequencyChart`](frontend/src/lib/stats/FrequencyChart.svelte) — график частоты боёв
- [`ResultsChart`](frontend/src/lib/stats/ResultsChart.svelte) — динамика побед/поражений
- [`TopTechniques`](frontend/src/lib/stats/TopTechniques.svelte) — топ используемых техник
- [`BodySilhouette`](frontend/src/lib/stats/BodySilhouette.svelte) × 2 — нанесённый и полученный урон
- [`HistoryTable`](frontend/src/lib/stats/HistoryTable.svelte) — таблица истории сходов с фильтрацией
- Единый `filteredBouts` derived-стор; клики на графики применяют фильтры к таблице

---

### 6.5. Компоненты плеера

#### [`frontend/src/lib/player/VideoPlayer.svelte`](frontend/src/lib/player/VideoPlayer.svelte)
HTML5 `<video>` элемент с расширенным управлением.
- **Props:** `src, speed, volume`, колбеки `ontimeupdate, ondurationchange, onplayingchange, onloopingchange, onfpschange`
- **Экспортируемые методы:** `seekTo(ms)`, `pause()`, `play()`, `togglePlay()`, `setLoop(start, end, autoPlay?)`, `toggleLoop()`, `stepForward()`, `stepBackward()`, `setSpeed(s)`, `setVolume(v)`
- **Функции:** цифровой зум (колесо мыши с индикатором), пан (drag при зуме), двойной клик = сброс зума, loop по диапазону, покадровый шаг через вычисление `1/fps`
- **CSS:** `transform: scale()` для зума, курсор при панорамировании

#### [`frontend/src/lib/player/Chat.svelte`](frontend/src/lib/player/Chat.svelte)
Чат с комментариями.
- **Props:** `videoId, comments, currentTime, highlightedId?`, колбеки `onseek, oncommentschange`
- **Состояния:** `text`, `replyTo`, `sending`, `editingId`, `editText`
- **Функции:**
  - `fmtMs(ms)` — форматирование миллисекунд в HH:MM:SS
  - `getReplyPreview(replyToId)` — превью текста родительского комментария
  - `submit()` — отправка (Enter или кнопка)
  - `startEdit(id) / submitEdit(id)` — редактирование своего комментария
  - `handleDelete(id)` — удаление (с confirm)
  - `handleReact(id, kind)` — лайк/дизлайк
- **Отображение:** рекурсивная структура тредов (ответы с отступом 16px); при изменении `highlightedId` скроллит к соответствующему сообщению и подсвечивает его анимацией
- **WebSocket:** слушает `new_comment` события для live-обновлений

#### [`frontend/src/lib/player/JudgingPanel.svelte`](frontend/src/lib/player/JudgingPanel.svelte)
Панель разметки сходов (левая панель).
- **Props:** `video, currentTime, playing`, колбеки `onboutschange, onseekrequest`
- **Состояния:** `bouts[]`, `fighterAId`, `fighterBId`, `startTime: number | null`
- **Экспортируемые методы:** `triggerMark()` — фиксирует начало/конец схода; `expandBout(id)` — разворачивает карточку схода
- **Функции:**
  - `saveFighters()` — назначает бойцов на видео
  - Авто-сохранение бойцов при изменении дропдауна
  - Отображение TOTAL SCORE в футере (`sum(score_a) : sum(score_b)`)
- **WebSocket:** слушает `update_bout` для live-обновлений
- Содержит список [`BoutCard`](frontend/src/lib/player/BoutCard.svelte)

#### [`frontend/src/lib/player/BoutCard.svelte`](frontend/src/lib/player/BoutCard.svelte)
Карточка отдельного схода.
- **Состояния:** свёрнута/развёрнута, dirty-флаг для несохранённых данных
- **Поля:** очки A/B (+/− кнопки), техника (дропдаун `<select>`), зона поражения ([`HitZonePicker`](frontend/src/lib/player/HitZonePicker.svelte)), результат (кастомный dropdown `position: fixed` для обхода overflow-clipping)
- **Tooltip техники:** при наведении на дропдаун показывает `description` техники во floating tooltip (`position: fixed`, рендерится вне overflow-контейнера)
- **Кнопки:** Save, Delete, Collapse
- **Внутренние функции:** `handleCollapse()` — с проверкой dirty-флага (не экспортируется)

#### [`frontend/src/lib/player/HitZonePicker.svelte`](frontend/src/lib/player/HitZonePicker.svelte)
Визуальный выбор зоны поражения на SVG-силуэте человека. **16 зон:** Голова, Шея, Плечо/Предплечье/Кисть (пр. и лев.), Тело, Таз, Бедро/Голень/Стопа (пр. и лев.). Значение хранится как `"ЗонаNазвание:x:y"` (координаты клика в SVG). Экспортирует константу `HIT_ZONES` (массив всех названий зон).

#### [`frontend/src/lib/player/Timeline.svelte`](frontend/src/lib/player/Timeline.svelte)
Нижняя панель управления воспроизведением.
- **Props:** `currentTime, duration, bouts, comments, fighterA, fighterB, playing, looping, speed, volume, fps`, колбеки: `onseek, onloop, onboutclick, oncommentclick, onplay, onstepback, onstepforward, onspeedchange, onvolumechange, onlooptoggle`
- **Три дорожки:**
  1. Метки комментариев (точки на временной шкале); клик → `onseek` + `oncommentclick`
  2. Основной прогресс-бар (клик/драг = перемотка)
  3. Трек сходов (цветные прямоугольники-сегменты с цветами бойцов)
- **Контролы:** Play/Pause, Step Back, Step Forward, Loop (toggle), Speed (0.1×–2.0×), Volume, Time (формат MM:SS:FFF с номером кадра)

---

### 6.6. Компоненты галереи

#### [`frontend/src/lib/gallery/Sidebar.svelte`](frontend/src/lib/gallery/Sidebar.svelte)
Боковая панель фильтров галереи.
- Список чекбоксов бойцов с количеством видео
- Виджет выбора диапазона дат (DateFrom / DateTo)
- Кнопка сброса фильтров

#### [`frontend/src/lib/gallery/VideoCard.svelte`](frontend/src/lib/gallery/VideoCard.svelte)
Карточка видео в галерее.
- Статичное превью (`/api/videos/{id}/previews/0`)
- Отображение: Боец A vs Боец B, счёт `X : Y`, дата
- Неразмеченное: маркер «Заполните данные»
- Клик: размеченное → переход в плеер, неразмеченное → модалка назначения бойцов
- Средняя кнопка мыши: открывает видео в новой вкладке

#### [`frontend/src/lib/gallery/VideoGrid.svelte`](frontend/src/lib/gallery/VideoGrid.svelte)
CSS Grid контейнер для карточек видео. Принимает список видео, рендерит [`VideoCard`](frontend/src/lib/gallery/VideoCard.svelte) для каждого.

---

### 6.7. Компоненты статистики

#### [`frontend/src/lib/stats/FighterSidebar.svelte`](frontend/src/lib/stats/FighterSidebar.svelte)
Компонент существует, но в `Stats.svelte` не используется. Выбор бойца реализован inline в hero-блоке.

#### [`frontend/src/lib/stats/QuickStats.svelte`](frontend/src/lib/stats/QuickStats.svelte)
**9 KPI-блоков:** Всего боёв (с тегами / всего), Всего сходов (ср. за бой), Винрейт по боям, Винрейт по сходам, Набрано очков, Пропущено очков, Попадаю чаще, Промахиваюсь, Пропускаю чаще.
- **Props:** `bouts, totalVideos?`

#### [`frontend/src/lib/stats/TopTechniques.svelte`](frontend/src/lib/stats/TopTechniques.svelte)
Топ техник бойца по частоте применения. Клик применяет фильтр по технике.

#### [`frontend/src/lib/stats/RecentOpponents.svelte`](frontend/src/lib/stats/RecentOpponents.svelte)
Горизонтальный список оппонентов с win/loss/total и balance. Wins/losses считаются по **боям** (по video_id), не по отдельным сходам. Клик применяет фильтр по оппоненту.

#### [`frontend/src/lib/stats/FrequencyChart.svelte`](frontend/src/lib/stats/FrequencyChart.svelte)
График частоты поединков (X = ISO-недели, Y = количество видео). Показывает **все недели** между первым и последним боем, включая пустые. Chart.js bar. Клик по столбцу фильтрует таблицу по неделе.

#### [`frontend/src/lib/stats/ResultsChart.svelte`](frontend/src/lib/stats/ResultsChart.svelte)
График динамики результатов (победа=1, ничья=0, поражение=−1 по видео). Процент побед в заголовке. Chart.js line с `cubicInterpolationMode: 'monotone'`. Клик по точке фильтрует таблицу по дате.

#### [`frontend/src/lib/stats/ScoreChart.svelte`](frontend/src/lib/stats/ScoreChart.svelte)
Линейный график прогресса по баллам (суммарные очки за тренировку). Chart.js.

#### [`frontend/src/lib/stats/BodySilhouette.svelte`](frontend/src/lib/stats/BodySilhouette.svelte)
SVG-силуэт человека в T-позе. Зоны кликабельны — клик устанавливает `zoneFilter`. Заливка бинарная (попадание/нет, не зависит от частоты). Рядом с силуэтом — легенда с количеством попаданий по зонам (имя и число вплотную).

#### [`frontend/src/lib/stats/HistoryTable.svelte`](frontend/src/lib/stats/HistoryTable.svelte)
Интерактивная таблица истории сходов:
- Колонки (переключаются через column picker): Видео (метка), Дата, Оппонент, Счёт, Мой приём, Мой рез., Моя зона, Приём опп., Рез. опп., Зона опп., →
- Сортировка по всем колонкам
- Фильтры: текстовые (оппонент, счёт), выпадающие `<select>` (Мой рез., Рез. опп., Мой приём, Приём опп., Моя зона, Зона опп., дата), выпадающий список бойцов
- Техника фильтруется по точному имени; зона — по имени зоны (из `HIT_ZONES`)
- Клик → переход к видео в плеере

---

### 6.8. Общие UI-компоненты

#### [`frontend/src/lib/ui/Header.svelte`](frontend/src/lib/ui/Header.svelte)
Глобальный хедер приложения.
- **Лево:** логотип + "Errant Fox" → клик → `#/gallery`
- **Центр:** навигация `[Видео] [Бойцы]`
- **Право:** кнопка поиска, аватар пользователя с dropdown-меню
- **Dropdown:** профиль, выход; Admin: «Создать бойца», «Техники»
- **Модальные окна:** [`CreateUserModal`](frontend/src/lib/admin/CreateUserModal.svelte), [`TechniquesModal`](frontend/src/lib/admin/TechniquesModal.svelte), [`ProfileModal`](frontend/src/lib/ui/ProfileModal.svelte), [`SearchPanel`](frontend/src/lib/ui/SearchPanel.svelte)
- **Функции:** `navigate(path)`, `logout()`, `toggleSearch()`, `handleClickOutside()`

#### [`frontend/src/lib/ui/ProfileModal.svelte`](frontend/src/lib/ui/ProfileModal.svelte)
Модальное окно редактирования профиля: username, display_name, цвет (color picker), пароль, аватар (загрузка файла).

#### [`frontend/src/lib/ui/SearchPanel.svelte`](frontend/src/lib/ui/SearchPanel.svelte)
Панель поиска по комментариям. Вызывает [`searchComments()`](frontend/src/lib/api/comments.ts), отображает результаты с возможностью перехода к видео и таймкоду.

---

### 6.9. Админ-компоненты

#### [`frontend/src/lib/admin/CreateUserModal.svelte`](frontend/src/lib/admin/CreateUserModal.svelte)
Модальное окно управления бойцами (полный CRUD): список всех бойцов с inline-редактированием (display_name, password, color, is_admin, avatar) + форма создания нового бойца. Вызывает [`createUser()`](frontend/src/lib/api/fighters.ts), [`patchUser()`](frontend/src/lib/api/fighters.ts), [`deleteUser()`](frontend/src/lib/api/fighters.ts), [`uploadUserAvatar()`](frontend/src/lib/api/fighters.ts).

#### [`frontend/src/lib/admin/TechniquesModal.svelte`](frontend/src/lib/admin/TechniquesModal.svelte)
Модальное окно управления техниками: список с возможностью добавить/переименовать/удалить. Вызывает [`getTechniques()`](frontend/src/lib/api/techniques.ts), [`createTechnique()`](frontend/src/lib/api/techniques.ts), [`renameTechnique()`](frontend/src/lib/api/techniques.ts), [`deleteTechnique()`](frontend/src/lib/api/techniques.ts).

---

## 7. База данных

**Движок:** SQLite (один файл). **ORM:** Diesel.

### Таблицы

#### `users`
| Колонка | Тип | Ограничения |
|---|---|---|
| `id` | TEXT | PK |
| `username` | TEXT | UNIQUE NOT NULL |
| `display_name` | TEXT | NOT NULL |
| `password_hash` | TEXT | NOT NULL |
| `is_admin` | BOOLEAN | NOT NULL DEFAULT false |
| `avatar_path` | TEXT | NULL |
| `color` | TEXT | NULL |
| `created_at` | TIMESTAMP | NOT NULL DEFAULT now |

#### `videos`
| Колонка | Тип | Ограничения |
|---|---|---|
| `id` | TEXT | PK |
| `seafile_path` | TEXT | UNIQUE NOT NULL |
| `fighter_a_id` | TEXT | FK→users.id NULL |
| `fighter_b_id` | TEXT | FK→users.id NULL |
| `date` | DATE | NOT NULL |
| `duration_ms` | INTEGER | NULL |
| `preview_count` | INTEGER | NOT NULL DEFAULT 0 |
| `created_at` | TIMESTAMP | NOT NULL DEFAULT now |

#### `techniques`
| Колонка | Тип | Ограничения |
|---|---|---|
| `id` | INTEGER | PK AUTOINCREMENT |
| `name` | TEXT | UNIQUE NOT NULL |
| `description` | TEXT | NULL |

#### `bouts`
| Колонка | Тип | Ограничения |
|---|---|---|
| `id` | INTEGER | PK AUTOINCREMENT |
| `video_id` | TEXT | FK→videos.id NOT NULL |
| `order_index` | INTEGER | NOT NULL |
| `time_start_ms` | INTEGER | NOT NULL |
| `time_end_ms` | INTEGER | NOT NULL |
| `score_a` | INTEGER | NOT NULL DEFAULT 0 |
| `score_b` | INTEGER | NOT NULL DEFAULT 0 |
| `technique_a_id` | INTEGER | FK→techniques.id NULL |
| `technique_b_id` | INTEGER | FK→techniques.id NULL |
| `hit_zone_a` | TEXT | NULL |
| `hit_zone_b` | TEXT | NULL |
| `result_a` | TEXT | NULL |
| `result_b` | TEXT | NULL |

#### `comments`
| Колонка | Тип | Ограничения |
|---|---|---|
| `id` | INTEGER | PK AUTOINCREMENT |
| `video_id` | TEXT | FK→videos.id NOT NULL |
| `author_id` | TEXT | FK→users.id NOT NULL |
| `timestamp_ms` | INTEGER | NOT NULL |
| `text` | TEXT | NOT NULL |
| `reply_to_id` | INTEGER | FK→comments.id NULL |
| `created_at` | TIMESTAMP | NOT NULL DEFAULT now |
| `edited_at` | TIMESTAMP | NULL |
| `bout_id` | INTEGER | FK→bouts.id NULL |

#### `comment_reactions`
| Колонка | Тип | Ограничения |
|---|---|---|
| `comment_id` | INTEGER | PK (composite), FK→comments.id |
| `user_id` | TEXT | PK (composite), FK→users.id |
| `kind` | TEXT | NOT NULL (`"like"` / `"dislike"`) |

### Миграции

| Файл | Содержание |
|---|---|
| [`migrations/0001_initial/`](backend/migrations/0001_initial/) | Создание таблиц users, videos, techniques, bouts, comments |
| [`migrations/0002_comment_reactions/`](backend/migrations/0002_comment_reactions/) | Добавление таблицы comment_reactions |
| [`migrations/0003_comment_bout_search/`](backend/migrations/0003_comment_bout_search/) | Добавление колонки bout_id в comments |
| [`migrations/0004_technique_description/`](backend/migrations/0004_technique_description/) | Добавление колонки description в techniques |

---

## 8. REST API: все эндпоинты

### Авторизация
| Метод | Путь | Хендлер | Аутентификация |
|---|---|---|---|
| POST | `/api/auth/login` | [`auth::login`](backend/src/api/auth.rs) | Нет |

### Текущий пользователь
| Метод | Путь | Хендлер | Аутентификация |
|---|---|---|---|
| GET | `/api/users/me` | [`auth::get_me`](backend/src/api/auth.rs) | JWT |
| PATCH | `/api/users/me` | [`users::patch_me`](backend/src/api/users.rs) | JWT |
| POST | `/api/users/me/avatar` | [`users::upload_avatar`](backend/src/api/users.rs) | JWT |
| GET | `/api/users/{id}/avatar` | [`users::get_avatar`](backend/src/api/users.rs) | JWT |

### Бойцы
| Метод | Путь | Хендлер | Аутентификация |
|---|---|---|---|
| GET | `/api/fighters` | [`users::list_fighters`](backend/src/api/users.rs) | JWT |
| GET | `/api/fighters/{id}/bouts` | [`users::fighter_bouts`](backend/src/api/users.rs) | JWT |

### Админ — пользователи
| Метод | Путь | Хендлер | Аутентификация |
|---|---|---|---|
| POST | `/api/admin/users` | [`users::create_user`](backend/src/api/users.rs) | Admin |
| PATCH | `/api/admin/users/{id}` | [`users::patch_admin_user`](backend/src/api/users.rs) | Admin |
| DELETE | `/api/admin/users/{id}` | [`users::delete_user`](backend/src/api/users.rs) | Admin |
| POST | `/api/admin/users/{id}/avatar` | [`users::upload_avatar_for`](backend/src/api/users.rs) | Admin |

### Техники
| Метод | Путь | Хендлер | Аутентификация |
|---|---|---|---|
| GET | `/api/techniques` | [`techniques::list_techniques`](backend/src/api/techniques.rs) | JWT |
| POST | `/api/admin/techniques` | [`techniques::create_technique`](backend/src/api/techniques.rs) | Admin |
| PATCH | `/api/admin/techniques/{id}` | [`techniques::patch_technique`](backend/src/api/techniques.rs) | Admin |
| DELETE | `/api/admin/techniques/{id}` | [`techniques::delete_technique`](backend/src/api/techniques.rs) | Admin |

### Видео
| Метод | Путь | Хендлер | Аутентификация |
|---|---|---|---|
| GET | `/api/videos` | [`videos::list_videos`](backend/src/api/videos.rs) | JWT |
| GET | `/api/videos/{id}` | [`videos::get_video`](backend/src/api/videos.rs) | JWT |
| PATCH | `/api/videos/{id}` | [`videos::patch_video`](backend/src/api/videos.rs) | JWT |
| GET | `/api/videos/{id}/stream` | [`videos::stream_video`](backend/src/api/videos.rs) | JWT |
| GET | `/api/videos/{id}/previews/{frame}` | [`videos::get_preview_frame`](backend/src/api/videos.rs) | JWT |

### Сходы
| Метод | Путь | Хендлер | Аутентификация |
|---|---|---|---|
| POST | `/api/bouts` | [`bouts::post_bout`](backend/src/api/bouts.rs) | JWT |
| PATCH | `/api/bouts/{id}` | [`bouts::patch_bout`](backend/src/api/bouts.rs) | JWT |
| DELETE | `/api/bouts/{id}` | [`bouts::delete_bout`](backend/src/api/bouts.rs) | JWT |

### Комментарии
| Метод | Путь | Хендлер | Аутентификация |
|---|---|---|---|
| POST | `/api/comments` | [`comments::post_comment`](backend/src/api/comments.rs) | JWT |
| GET | `/api/comments/search` | [`comments::search_comments`](backend/src/api/comments.rs) | JWT |
| PATCH | `/api/comments/{id}` | [`comments::patch_comment`](backend/src/api/comments.rs) | JWT |
| DELETE | `/api/comments/{id}` | [`comments::delete_comment`](backend/src/api/comments.rs) | JWT |
| POST | `/api/comments/{id}/react` | [`comments::react_comment`](backend/src/api/comments.rs) | JWT |
| DELETE | `/api/comments/{id}/react` | [`comments::delete_react`](backend/src/api/comments.rs) | JWT |

---

## 9. WebSocket API

### Подключение: `ws://host/ws`

**Аутентификация:** первое сообщение от клиента — JSON `{"token": "eyJ..."}`.

**Фильтрация:** клиент может отправить `{"watching": "video_id"}` для получения событий только этого видео.

### События от сервера (JSON)

| Тип (`"type"`) | Поле данных | Когда |
|---|---|---|
| `"new_comment"` | [`WsComment`](backend/src/ws.rs) | Другой пользователь оставил комментарий |
| `"update_bout"` | [`WsBout`](backend/src/ws.rs) | Создан/изменён сход |
| `"new_video"` | `{id, date, preview_url}` | Seafile sync обнаружил новое видео |

---

## 10. Инфраструктура (Docker / Nginx)

### [`infra/docker-compose.yml`](infra/docker-compose.yml)

Два сервиса:

**`backend`:**
- Собирается из [`Dockerfile.backend`](infra/Dockerfile.backend)
- Монтирует `../data` → `/data` (БД, превью, аватары)
- Читает `.env` из корня проекта
- Переменные: `DATABASE_URL=/data/db/errant_fox.db`, `PREVIEWS_DIR=/data/previews`, `AVATARS_DIR=/data/avatars`

**`frontend`:**
- Собирается из [`Dockerfile.frontend`](infra/Dockerfile.frontend)
- Nginx на порту 80
- Traefik labels для HTTPS (Let's Encrypt) на домене `errantfox.aat-terra.ru`
- Сеть `proxy` (внешняя, для Traefik)

### [`infra/Dockerfile.backend`](infra/Dockerfile.backend)
Multi-stage сборка:
1. `rust:1.88-alpine` — сборка релиза (cargo build --release)
2. `alpine:3.20` — рантайм с `ca-certificates`, `ffmpeg`. Запускает `./errant_fox`

### [`infra/Dockerfile.frontend`](infra/Dockerfile.frontend)
Multi-stage сборка:
1. `node:20-alpine` — `npm ci` + `npm run build` (Vite)
2. `nginx:alpine` — копирует `dist/` + `nginx.conf`

### [`infra/nginx.conf`](infra/nginx.conf)
Nginx конфигурация:
- SPA fallback: `try_files $uri $uri/ /index.html`
- Прокси `/api/` → `http://backend:8080`
- Прокси `/ws` → `http://backend:8080` (с Upgrade для WebSocket, `proxy_read_timeout 3600s`)

---

## 11. Документация в docs/

Дополнительные документы проекта:

| Файл | Содержание |
|---|---|
| [`docs/requirements.md`](docs/requirements.md) | Полное техническое задание: обзор, пользователи, интеграция с Seafile, модель данных, все экраны, нефункциональные требования |
| [`docs/architecture.md`](docs/architecture.md) | Архитектура: распределение функций между фронтендом и бэкендом, схема взаимодействия, список всех экранов с разбивкой по компонентам |
| [`docs/api.md`](docs/api.md) | REST API reference: все эндпоинты с примерами запросов/ответов |
| [`docs/database.md`](docs/database.md) | Схема БД: таблицы, колонки, индексы, ER-диаграмма |
| [`docs/implementation_plan.md`](docs/implementation_plan.md) | Пошаговый план реализации (фазы 0-7) — исторический документ |
| [`deploy.md`](deploy.md) | Инструкция по деплою на TrueNAS SCALE |

---

## 12. Архив (\_archive/)

Старый код проекта Clapshot (предшественник Errant Fox). **Не используется в текущем проекте.** Сохранён для истории.

- **`_archive/backend_rust/`** — старый Rust-бэкенд с gRPC, Diesel (другие модели), video pipeline
- **`_archive/frontend_noodl/`** — старый фронтенд на платформе Noodl (low-code)
- **`_archive/backend_rust/protobuf/`** — Protobuf-схемы из старой архитектуры

---

> **Для AI-ассистента:** этот файл — главный источник истины о проекте. При анализе любой задачи начинай с него. Для деталей реализации конкретных функций переходи по ссылкам на файлы исходного кода.
