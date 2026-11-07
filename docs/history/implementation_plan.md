# Errant Fox — План реализации

> **Статус:** Фазы 0–7 реализованы. Документ отражает фактическую реализацию с указанием расхождений с изначальным планом.
>
> Опирается на: [architecture.md](architecture.md) · [api.md](api.md) · [database.md](database.md)

---

## Структура папок (фактическая)

```
Errant Fox/
├── backend/                  ← Rust сервер (чистая реализация)
│   ├── src/
│   │   ├── main.rs           ← Точка входа, CORS, запуск sync + сервера
│   │   ├── config.rs         ← Конфиг из env-переменных (8 полей)
│   │   ├── errors.rs         ← AppError enum (Unauthorized/Forbidden/NotFound/BadRequest/Internal)
│   │   ├── state.rs          ← AppState (db pool, jwt_secret, dirs, seafile, ws_hub)
│   │   ├── db/
│   │   │   ├── mod.rs        ← init_pool + run_migrations
│   │   │   ├── schema.rs     ← Diesel schema (6 таблиц)
│   │   │   └── models.rs     ← Queryable/Insertable структуры
│   │   ├── api/
│   │   │   ├── mod.rs        ← Axum router (16+ маршрутов)
│   │   │   ├── auth.rs       ← login, get_me, patch_me, JWT generate/verify
│   │   │   ├── users.rs      ← fighters, me, avatar, admin users CRUD
│   │   │   ├── videos.rs     ← list, get, patch, stream, previews
│   │   │   ├── bouts.rs      ← post, patch, delete
│   │   │   ├── comments.rs   ← post, patch, delete, react, search
│   │   │   └── techniques.rs ← list, create, patch, delete
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   └── auth.rs       ← CurrentUser FromRequestParts extractor
│   │   ├── seafile.rs        ← HTTP-клиент к Seafile REST API
│   │   ├── sync.rs           ← Фоновая sync задача (tokio::spawn, интервал 60 сек)
│   │   ├── previews.rs       ← FFmpeg генерация 10 превью-кадров JPG
│   │   ├── moov.rs           ← Парсинг MP4 moov atom (FPS)
│   │   └── ws.rs             ← WebSocket handler + WsHub (broadcast)
│   ├── migrations/
│   │   ├── 0001_initial/     ← 5 таблиц + индексы
│   │   ├── 0002_comment_reactions/
│   │   ├── 0003_comment_bout_search/
│   │   ├── 0004_technique_description/
│   │   └── 0005_video_fps/
│   ├── Cargo.toml
│   └── .env.example
├── frontend/                 ← Svelte 5 + TypeScript + Vite
│   ├── src/
│   │   ├── main.ts
│   │   ├── App.svelte        ← Роутинг, темы, initStores()
│   │   ├── app.css           ← CSS-переменные, светлая/тёмная темы
│   │   ├── stores.ts         ← Svelte stores (token, currentUser, techniques, fighters)
│   │   ├── lib/
│   │   │   ├── api/          ← Клиент к REST API
│   │   │   │   ├── client.ts
│   │   │   │   ├── types.ts
│   │   │   │   ├── auth.ts, bouts.ts, comments.ts, fighters.ts, techniques.ts, videos.ts
│   │   │   ├── player/       ← VideoPlayer + Timeline + JudgingPanel + BoutCard + HitZonePicker + Chat + moov.ts
│   │   │   ├── gallery/      ← VideoCard + VideoGrid + Sidebar
│   │   │   ├── stats/        ← FighterSidebar, HistoryTable, QuickStats, TopTechniques, FrequencyChart,
│   │   │   │                   ResultsChart, ScoreChart, RadarChart, BodySilhouette, RecentOpponents
│   │   │   ├── admin/        ← CreateUserModal + TechniquesModal
│   │   │   └── ui/           ← Header + ProfileModal + SearchPanel
│   │   └── routes/           ← Auth.svelte, Gallery.svelte, Stats.svelte, Player.svelte
│   ├── package.json
│   └── vite.config.ts
├── infra/                    ← Docker-конфигурация
│   ├── docker-compose.yml
│   ├── docker-compose.local.yml
│   ├── Dockerfile.backend
│   ├── Dockerfile.frontend
│   └── nginx.conf
├── docs/                     ← Документация
└── deploy.md                 ← Инструкция по деплою
```

---

## Фаза 0 — Подготовка проекта ✅

**Результат:** новая структура папок, настроенный backend и frontend.

### Выполнено:
- [x] Создан `backend/` — новый Rust-проект (Axum 0.8 + Diesel 2.3)
- [x] Создан `frontend/` — новый Svelte 5 проект (не перенос из TrueNAS, написан заново)
- [x] Создан `backend/.env.example`
- [x] Создана `infra/` с Docker-конфигурацией (отклонение: не в корне, а в отдельной папке)

**Переменные окружения (`backend/.env`):**
```
DATABASE_URL=./errant_fox.sqlite
JWT_SECRET=your-secret-key-here
SEAFILE_URL=http://localhost:8082
SEAFILE_TOKEN=your-seafile-api-token
PREVIEWS_DIR=./data/previews
AVATARS_DIR=./data/avatars
SERVER_PORT=8080
FRONTEND_ORIGIN=http://localhost:5173
```

---

## Фаза 1 — Backend: Фундамент ✅

**Результат:** работающий сервер с авторизацией и базой данных.

### Зависимости (`Cargo.toml` — фактические):
```toml
axum = { version = "0.8", features = ["multipart", "ws"] }
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.6", features = ["cors"] }
diesel = { version = "2.3", features = ["sqlite", "r2d2", "chrono"] }
diesel_migrations = "2.3"
libsqlite3-sys = { version = "0.30", features = ["bundled"] }
r2d2 = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
jsonwebtoken = "9"
bcrypt = "0.15"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "stream"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1"
regex = "1"
tokio-tungstenite = "0.26"
```

> Отклонения от плана: добавлены `tower-http` для CORS, `regex` для парсинга дат, `anyhow` для moov.rs. `reqwest` использует `rustls-tls` вместо native-tls.

### Выполнено:
- [x] `src/config.rs` — Config из env (8 полей, паника при отсутствии)
- [x] `src/db/` — Diesel pool (r2d2), 0001_initial миграция, модели
- [x] `src/middleware/auth.rs` — CurrentUser Axum extractor
- [x] `src/api/auth.rs` — login (bcrypt verify), make_token, verify_token, get_me, patch_me
- [x] `src/main.rs` — Axum router, CORS, запуск sync, serve
- [x] Цвета: палитра из 12 цветов, генерируется детерминированно из user.id

---

## Фаза 2 — Backend: API + Seafile ✅

**Результат:** все REST-эндпоинты работают, видео появляются из Seafile, FPS извлекается.

### Выполнено:
- [x] `src/api/users.rs` — GET/PATCH me, POST avatar, GET fighters, GET fighter bouts, admin CRUD
- [x] `src/api/techniques.rs` — GET list, POST create, PATCH, DELETE
- [x] `src/seafile.rs` — HTTP-клиент (list_folders, list_files, get_download_url)
- [x] `src/sync.rs` — периодический опрос 60 сек, парсинг даты, FPS через moov.rs, INSERT videos
- [x] `src/moov.rs` — парсинг MP4 moov atom: timescale / sample_delta = FPS
- [x] `src/api/videos.rs` — GET list (фильтры), GET by id, PATCH tag, GET stream, GET previews
- [x] `src/previews.rs` — FFmpeg: 10 кадров JPG через `select` filter
- [x] `src/api/bouts.rs` — POST, PATCH, DELETE + WS broadcast
- [x] `src/api/comments.rs` — POST, PATCH, DELETE + auto bout_id + WS broadcast
- [x] `src/ws.rs` — WsHub, handler, фильтрация по watching

### Дополнительно реализовано (не в изначальном плане):
- [x] `POST/DELETE /api/comments/:id/react` — реакции на комментарии (таблица `comment_reactions`)
- [x] `GET /api/comments/search` — полнотекстовый поиск по комментариям
- [x] `PATCH /api/admin/users/:id` — редактирование пользователей админом
- [x] `POST /api/admin/users/:id/avatar` — загрузка аватара админом
- [x] `PATCH /api/admin/techniques/:id` — редактирование названия и описания техники
- [x] `GET /api/videos/:id` — возвращает `fps` поле
- [x] `GET /api/videos/:id` — комментарии включают `likes`, `dislikes`, `my_reaction`, `bout_id`

---

## Фаза 3 — Frontend: Фундамент ✅

**Результат:** Svelte 5 приложение, авторизация, роутинг, stores, API-клиент.

### Выполнено:
- [x] `src/lib/api/client.ts` — apiFetch<T> с Bearer токеном, 401 → редирект
- [x] `src/lib/api/types.ts` — User, Fighter, FighterBout, Video, VideoFull, Bout, Comment, Technique, VideoShort, SearchResult
- [x] `src/lib/api/auth.ts` — login, getMe
- [x] `src/lib/api/videos.ts` — getVideos, getVideo, patchVideo, getStreamUrl
- [x] `src/lib/api/bouts.ts` — createBout, updateBout, deleteBout
- [x] `src/lib/api/comments.ts` — create, update, delete, react, deleteReact, searchComments
- [x] `src/lib/api/fighters.ts` — getFighters, getFighterBouts
- [x] `src/lib/api/techniques.ts` — get, create, patch, rename, delete
- [x] `src/stores.ts` — token, currentUser, techniques, fighters + initStores()
- [x] `src/routes/Auth.svelte` — форма логина с тёмно-синим фоном
- [x] `src/App.svelte` — hash-роутинг, темы (data-theme), initStores()
- [x] `src/lib/ui/Header.svelte` — логотип, навигация, поиск, тема, аватар + dropdown

---

## Фаза 4 — Галерея ✅

**Результат:** сетка карточек с фильтрами и scrub-превью.

### Выполнено:
- [x] `src/lib/gallery/VideoCard.svelte` — карточка 16:9, hover scrub, маркер неразмеченного
- [x] `src/lib/gallery/VideoGrid.svelte` — CSS-grid, lazy загрузка превью
- [x] `src/lib/gallery/Sidebar.svelte` — чекбоксы бойцов, дата-пикеры
- [x] `src/routes/Gallery.svelte` — реактивная фильтрация, TagModal inline, WS new_video

---

## Фаза 5 — Видео-плеер ✅

**Результат:** полноценный плеер с судейской панелью, чатом и таймлайном.

### Выполнено:
- [x] `src/lib/player/VideoPlayer.svelte` — HTML5 video + zoom + pan
- [x] `src/lib/player/moov.ts` — клиентский MP4 FPS-парсер (первые 2 МБ через Range)
- [x] `src/lib/player/Timeline.svelte` — прогресс-бар, маркеры, трек бутов
- [x] `src/lib/player/JudgingPanel.svelte` — START/FINISH, список карточек, TOTAL SCORE
- [x] `src/lib/player/BoutCard.svelte` — форма схода (очки, техника, HitZonePicker, результат)
- [x] `src/lib/player/HitZonePicker.svelte` — SVG-силуэт с 16 зонами, координаты x:y
- [x] `src/lib/player/Chat.svelte` — список, ответы, лайки/дизлайки, поиск
- [x] `src/routes/Player.svelte` — layout: JudgingPanel | VideoPlayer | Chat + Timeline

> Отклонения от плана: VideoPlayer написан заново на Svelte 5 (runes), а не портирован из TrueNAS. HitZonePicker — 16 зон вместо 6. Добавлены реакции (лайк/дизлайк).

---

## Фаза 6 — Статистика ✅

**Результат:** дашборд бойца с графиками, силуэтами и таблицей.

### Выполнено:
- [x] `src/lib/stats/FighterSidebar.svelte` — список бойцов
- [x] `src/lib/stats/HistoryTable.svelte` — сортировка/фильтрация по всем столбцам
- [x] `src/lib/stats/QuickStats.svelte` — 3 блока (groupBy + max)
- [x] `src/lib/stats/TopTechniques.svelte` — таблица техник с % успешности
- [x] `src/lib/stats/FrequencyChart.svelte` — Chart.js bar (недели)
- [x] `src/lib/stats/ResultsChart.svelte` — Chart.js line (победы/поражения)
- [x] `src/lib/stats/ScoreChart.svelte` — Chart.js line (очки)
- [x] `src/lib/stats/RadarChart.svelte` — Chart.js radar (7 осей результатов)
- [x] `src/lib/stats/BodySilhouette.svelte` — SVG силуэт, заливка 6 зон
- [x] `src/lib/stats/RecentOpponents.svelte` — список противников с win/loss
- [x] Все компоненты используют единый filtered-массив из HistoryTable

> Дополнительно: RadarChart и RecentOpponents не были в изначальном плане. Chart.js загружается динамически (`import('chart.js')`).

---

## Фаза 7 — Администрирование ✅

**Результат:** Admin UI для управления пользователями и техниками.

### Выполнено:
- [x] `src/lib/admin/CreateUserModal.svelte` — форма с полями username, display_name, password, is_admin, color
- [x] `src/lib/admin/TechniquesModal.svelte` — список + добавить/изменить/удалить, description
- [x] `src/lib/ui/ProfileModal.svelte` — профиль текущего пользователя
- [x] `src/lib/ui/SearchPanel.svelte` — поиск по комментариям

---

## Фаза 8 — Полировка и деплой ✅

**Результат:** продакшн-готовое приложение с Docker.

### Выполнено:
- [x] Состояния загрузки (skeleton) в галерее и статистике
- [x] Обработка ошибок: apiFetch показывает сообщения, WS переподключается
- [x] Тёмная/светлая темы (CSS-переменные, data-theme)
- [x] Адаптивная вёрстка (mobile-friendly)
- [x] Dockerfile (multi-stage) и docker-compose конфигурация
- [x] deploy.md — инструкция по деплою на TrueNAS
- [x] Миграции применяются автоматически при старте бэкенда

---

## Фаза 9 — Модуль ИИ-разметки, Публичный шаринг и Расширения ✅

**Результат:** интеграция Faster-Whisper, акустическое уточнение пиков, пакетный анализ, гостевые ссылки и учет ролей.

### Выполнено:
- [x] Разработка микросервиса `whisper-service` (Python + FastAPI + Faster-Whisper int8 CTranslate2)
- [x] Алгоритм **Acoustic Peak Refinement** (анализ RMS энергии волновой формы около выкрика)
- [x] Пакетная разметка (`Batch AI Modal`) и система отмены заданий
- [x] Публичные ссылки и шаринг видео (`POST /api/videos/:id/share`, `/api/shared/*`)
- [x] Гостевые комментарии со свободным указанием никнейма
- [x] Миграции 0006–0012 (`vk_id`, `bout_history`, `guest_nickname`, `role`, `is_ai_labeled`, `is_analyzing`, `is_ai`)
- [x] Инспектор распознавания сходов (`GET /api/videos/:id/transcript`)
- [x] Реактивные обновления Timeline и Плеера на Svelte 5

---

## Сводка реализованных эндпоинтов

| Метод | Путь | Файл |
|---|---|---|
| POST | `/api/auth/login` | `api/auth.rs` |
| GET | `/api/users/me` | `api/auth.rs` |
| PATCH | `/api/users/me` | `api/auth.rs` / `api/users.rs` |
| POST | `/api/users/me/avatar` | `api/users.rs` |
| GET | `/api/users/:id/avatar` | `api/users.rs` |
| GET | `/api/fighters` | `api/users.rs` |
| GET | `/api/fighters/:id/bouts` | `api/users.rs` |
| POST | `/api/admin/users` | `api/users.rs` |
| PATCH | `/api/admin/users/:id` | `api/users.rs` |
| POST | `/api/admin/users/:id/avatar` | `api/users.rs` |
| DELETE | `/api/admin/users/:id` | `api/users.rs` |
| GET | `/api/techniques` | `api/techniques.rs` |
| POST | `/api/admin/techniques` | `api/techniques.rs` |
| PATCH | `/api/admin/techniques/:id` | `api/techniques.rs` |
| DELETE | `/api/admin/techniques/:id` | `api/techniques.rs` |
| GET | `/api/videos` | `api/videos.rs` |
| GET | `/api/videos/:id` | `api/videos.rs` |
| PATCH | `/api/videos/:id` | `api/videos.rs` |
| GET | `/api/videos/:id/stream` | `api/videos.rs` |
| GET | `/api/videos/:id/previews/:frame` | `api/videos.rs` |
| POST | `/api/videos/:id/ai-label` | `api/videos.rs` |
| POST | `/api/videos/:id/cancel-analysis` | `api/videos.rs` |
| GET | `/api/videos/:id/transcript` | `api/videos.rs` |
| POST | `/api/videos/:id/share` | `api/videos.rs` |
| GET | `/api/shared/videos/:id` | `api/videos.rs` |
| POST | `/api/shared/videos/:id/comments` | `api/videos.rs` |
| GET | `/api/shared/videos/:id/download` | `api/videos.rs` |
| GET | `/api/shared/bouts/:id/download` | `api/bouts.rs` |
| GET | `/api/admin/videos/sync-check` | `api/videos.rs` |
| POST | `/api/admin/videos/sync-clean` | `api/videos.rs` |
| POST | `/api/admin/videos/import` | `api/videos.rs` |
| POST | `/api/bouts` | `api/bouts.rs` |
| PATCH | `/api/bouts/:id` | `api/bouts.rs` |
| DELETE | `/api/bouts/:id` | `api/bouts.rs` |
| POST | `/api/comments` | `api/comments.rs` |
| PATCH | `/api/comments/:id` | `api/comments.rs` |
| DELETE | `/api/comments/:id` | `api/comments.rs` |
| POST | `/api/comments/:id/react` | `api/comments.rs` |
| DELETE | `/api/comments/:id/react` | `api/comments.rs` |
| GET | `/api/comments/search` | `api/comments.rs` |
| GET | `/ws` | `ws.rs` |

