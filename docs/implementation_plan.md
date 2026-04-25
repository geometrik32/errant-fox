# Errant Fox — План реализации

> Опирается на: [architecture.md](architecture.md) · [api.md](api.md) · [database.md](database.md)

---

## Структура папок (целевая)

```
Errant Fox/
├── backend/                  ← Rust сервер (новая чистая папка)
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs         ← Конфиг из env-переменных
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs     ← Генерируется Diesel
│   │   │   ├── models.rs     ← Rust-структуры таблиц
│   │   │   └── migrations/   ← SQL-миграции
│   │   ├── api/
│   │   │   ├── mod.rs        ← Axum router
│   │   │   ├── auth.rs       ← POST /api/auth/login
│   │   │   ├── users.rs      ← /api/users/me, /api/fighters
│   │   │   ├── videos.rs     ← /api/videos
│   │   │   ├── bouts.rs      ← /api/bouts
│   │   │   ├── comments.rs   ← /api/comments
│   │   │   └── techniques.rs ← /api/techniques
│   │   ├── seafile.rs        ← HTTP-клиент к Seafile API
│   │   ├── previews.rs       ← FFmpeg генерация превью кадров
│   │   ├── ws.rs             ← WebSocket live-события
│   │   └── middleware/
│   │       └── auth.rs       ← JWT middleware
│   ├── Cargo.toml
│   └── .env.example
├── frontend/                 ← Svelte 5 (перенос из TrueNAS/src/client/)
│   ├── src/
│   │   ├── lib/
│   │   │   ├── api/          ← Клиент к нашему REST API
│   │   │   ├── player/       ← VideoPlayer + Timeline + Chat
│   │   │   ├── gallery/      ← Галерея + фильтры
│   │   │   ├── stats/        ← Статистика + графики
│   │   │   └── ui/           ← Переиспользуемые компоненты
│   │   ├── routes/           ← Страницы (Auth, Gallery, Stats, Player)
│   │   ├── stores.ts         ← Глобальный стейт (Svelte stores)
│   │   └── types.ts          ← TypeScript-типы
│   ├── package.json
│   └── vite.config.ts
├── docs/                     ← Вся документация
└── docker-compose.yml        ← Деплой
```

---

## Фаза 0 — Подготовка проекта
**Цель:** чистая структура, настроенный git, готовые инструменты сборки.

### Задачи:
- [ ] Создать папку `backend/` — новый Rust-проект (`cargo new`)
- [ ] Перенести `frontend/` из `TrueNAS/src/client/` как основу
- [ ] Удалить из фронтенда: gRPC/protobuf зависимости, clapshot-специфичные компоненты
- [ ] Создать `backend/.env.example` с переменными окружения
- [ ] Проверить что `frontend/` запускается (`npm run dev`)
- [ ] Проверить что `backend/` компилируется (`cargo build`)

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

## Фаза 1 — Backend: Фундамент
**Цель:** работающий сервер с авторизацией и базой данных.

### Зависимости (`Cargo.toml`):
```toml
axum = { version = "0.8", features = ["multipart"] }
tokio = { version = "1", features = ["full"] }
diesel = { version = "2.3", features = ["sqlite", "r2d2", "chrono"] }
diesel_migrations = "2.3"
libsqlite3-sys = { version = "0.30", features = ["bundled"] }
r2d2 = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
jsonwebtoken = "9"
bcrypt = "0.15"
reqwest = { version = "0.12", features = ["json"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1"
tokio-tungstenite = "0.26"
```

### Задачи:
- [ ] `src/config.rs` — структура конфига, читает из `.env`
- [ ] `src/db/` — Diesel pool, 5 таблиц, первая миграция
- [ ] `src/middleware/auth.rs` — JWT extractor для Axum
- [ ] `src/api/auth.rs` — POST /api/auth/login (bcrypt + JWT)
- [ ] `src/main.rs` — Axum router, CORS, запуск сервера
- [ ] Проверить: login возвращает токен, токен проходит middleware

---

## Фаза 2 — Backend: API + Seafile
**Цель:** все REST-эндпоинты работают, видео появляются из Seafile.

### Задачи:
- [ ] `src/api/users.rs` — GET /api/users/me, PATCH, POST avatar, GET /api/fighters
- [ ] `src/api/techniques.rs` — GET /api/techniques, POST/DELETE (admin)
- [ ] `src/seafile.rs` — HTTP-клиент: список папок, список файлов, временная ссылка
- [ ] Seafile sync задача — периодический опрос каждые 60 сек (tokio::spawn)
- [ ] `src/api/videos.rs` — GET /api/videos (с фильтрами), GET /api/videos/:id, PATCH
- [ ] `src/api/videos.rs` — GET /api/videos/:id/stream (возвращает seafile URL)
- [ ] `src/previews.rs` — FFmpeg: скачать 10 кадров равномерно по видео, сохранить PNG
- [ ] `src/api/videos.rs` — GET /api/videos/:id/previews/:frame
- [ ] `src/api/bouts.rs` — POST, PATCH, DELETE /api/bouts
- [ ] `src/api/comments.rs` — POST, PATCH, DELETE /api/comments
- [ ] `src/api/admin.rs` — POST/DELETE /api/admin/users, techniques
- [ ] `src/ws.rs` — WebSocket hub, broadcast 3 событий
- [ ] Проверить каждый эндпоинт через curl или REST-клиент

---

## Фаза 3 — Frontend: Фундамент
**Цель:** приложение запускается, авторизация работает, роутинг настроен.

### Задачи:
- [ ] `src/lib/api/client.ts` — fetch-обёртка: добавляет Bearer токен, обрабатывает ошибки
- [ ] `src/lib/api/auth.ts` — login(), logout(), getCurrentUser()
- [ ] `src/lib/api/videos.ts` — getVideos(), getVideo(), patchVideo()
- [ ] `src/lib/api/bouts.ts`, `comments.ts`, `fighters.ts`, `techniques.ts`
- [ ] `src/stores.ts` — currentUser, token, techniques (кешируются)
- [ ] `src/types.ts` — TypeScript-интерфейсы под каждый API-ответ
- [ ] `src/routes/Auth.svelte` — форма логина
- [ ] `src/routes/` — роутинг: `/` → Gallery, `/player/:id` → Player, `/stats` → Stats
- [ ] `src/lib/ui/Header.svelte` — навигация, аватар, дропдаун
- [ ] WebSocket: connect после логина, обработчик 3 событий
- [ ] Проверить: логин → переход в галерею → выход

---

## Фаза 4 — Галерея
**Цель:** список видео с фильтрами, scrub-превью, модал для неразмеченных.

### Задачи:
- [ ] `src/lib/gallery/VideoCard.svelte` — карточка: превью, имена бойцов, счёт, маркер
- [ ] `src/lib/gallery/VideoCard.svelte` — hover scrub (mousemove → меняет кадр 0..9)
- [ ] `src/lib/gallery/VideoGrid.svelte` — сетка карточек, пагинация или lazy-load
- [ ] `src/lib/gallery/Sidebar.svelte` — чекбоксы бойцов + дата-пикер
- [ ] Реактивная фильтрация: изменение фильтра → новый GET /api/videos?...
- [ ] `src/lib/gallery/TagModal.svelte` — модал для выбора бойцов неразмеченного видео
- [ ] Проверить: фильтры работают, scrub плавный, тег сохраняется

---

## Фаза 5 — Видео-плеер
**Цель:** полноценный плеер со всеми элементами управления, судейством и чатом.

### Задачи:

**Плеер (переносим из TrueNAS):**
- [ ] `src/lib/player/VideoPlayer.svelte` — порт из TrueNAS, убрать Clapshot-специфику
- [ ] `src/lib/player/video-decoder/` — перенести HybridVideoDecoder, Html5, Mediabunny
- [ ] Подключить `stream_url` из GET /api/videos/:id/stream
- [ ] Проверить: play/pause, seek, зум, покадровая навигация Z/X, скорость

**Таймлайн:**
- [ ] `src/lib/player/Timeline.svelte` — прогресс-бар + маркеры комментов + трек бутов
- [ ] Клик на бут-сегмент → установить loop диапазон
- [ ] Клик на маркер комментария → seek + пауза

**Судейская панель:**
- [ ] `src/lib/player/JudgingPanel.svelte` — дропдауны бойцов, START/FINISH
- [ ] `src/lib/player/BoutCard.svelte` — форма схода: очки, техника, зона, результат
- [ ] Логика dirty-state: диалог при закрытии несохранённой карточки
- [ ] Сохранение через PATCH /api/bouts/:id
- [ ] Live-обновление: WS событие `update_bout` → обновить список

**Чат:**
- [ ] `src/lib/player/Chat.svelte` — список сообщений + поле ввода
- [ ] Клик тайм-кода → seek
- [ ] Live-обновление: WS событие `new_comment` → добавить в список

---

## Фаза 6 — Статистика
**Цель:** полный дашборд бойца с интерактивными фильтрами.

### Задачи:
- [ ] `src/lib/stats/FighterSidebar.svelte` — список бойцов для переключения
- [ ] Загрузка данных: GET /api/fighters/:id/bouts → один раз, все вычисления на фронте
- [ ] `src/lib/stats/HistoryTable.svelte` — таблица с сортировкой/фильтрацией по столбцам
- [ ] Реактивный `filteredBouts` store — все компоненты подписаны на него
- [ ] `src/lib/stats/QuickStats.svelte` — 3 быстрых блока (groupBy + max)
- [ ] `src/lib/stats/FrequencyChart.svelte` — Chart.js: недели × количество боёв
- [ ] `src/lib/stats/ResultsChart.svelte` — Chart.js: хронология побед/поражений
- [ ] `src/lib/stats/ScoreChart.svelte` — Chart.js: прогресс очков по тренировкам
- [ ] `src/lib/stats/BodySilhouette.svelte` — SVG силуэт + динамическая заливка зон
- [ ] Проверить: фильтр таблицы → моментально обновляет всё

---

## Фаза 7 — Администрирование
**Цель:** Admin может создавать пользователей и управлять техниками.

### Задачи:
- [ ] `src/lib/admin/CreateUserModal.svelte` — форма создания пользователя
- [ ] `src/lib/admin/TechniquesModal.svelte` — список техник + добавить/удалить
- [ ] Пункты в Header dropdown для Admin
- [ ] Проверить: создать пользователя, он появляется в дропдаунах и фильтрах

---

## Фаза 8 — Полировка и деплой
**Цель:** продакшн-готовое приложение на TrueNAS.

### Задачи:
- [ ] Состояния загрузки (skeleton) везде где данные подгружаются
- [ ] Обработка ошибок: потеря соединения, Seafile недоступен
- [ ] Мобильная адаптация (плеер сложный — минимум читаемость галереи и статистики)
- [ ] `docker-compose.yml` — backend + nginx (фронт собирается в static)
- [ ] `nginx.conf` — проксирование `/api` и `/ws` на бэкенд, статика фронта
- [ ] Инструкция по деплою на TrueNAS
- [ ] Финальное тестирование: несколько пользователей одновременно, live-обновления

---

## Порядок работы с Claude Code в VS Code

### Настройки режимов

| Режим | Когда использовать |
|---|---|
| **Plan mode** | Перед началом каждой фазы, при неопределённости |
| **Edit automatically** | Реализация конкретных задач с понятным ТЗ |
| **Ask before edits** | Миграции БД, конфиги, изменения существующего кода |
| **Effort: High** | Сложные многофайловые задачи (новый модуль, сложный компонент) |
| **Effort: Medium** | Точечные изменения, доработки, исправления |

---

### Как читать промпты ниже

Каждый промпт содержит три списка файлов:

- **📖 Прочитать** — Claude должен прочитать эти файлы перед работой (контекст и справка)
- **✏️ Создать / изменить** — что будет создано или изменено в результате
- **🚫 Не трогать** — файлы которые должны остаться нетронутыми

Копируй промпт целиком и вставляй в поле ввода Claude Code.

---

### Промпты по фазам

---

#### Фаза 0 — Подготовка проекта

**Режим: Edit automatically | Effort: High**

📖 Прочитать:
- `docs/implementation_plan.md` (раздел "Структура папок" и "Фаза 0")
- `docs/database.md`

✏️ Создать:
- `backend/Cargo.toml`
- `backend/src/main.rs`
- `backend/src/config.rs`
- `backend/src/db/mod.rs`
- `backend/src/api/mod.rs`
- `backend/src/middleware/auth.rs` (пустой файл-заглушка)
- `backend/.env.example`

🚫 Не трогать:
- `frontend_noodl/` — старый фронтенд, не менять
- `backend_rust/` — старый бэкенд, не менять
- `docs/`

```
Проект: Errant Fox — HEMA видео-аналитика.
Стек backend: Rust + Axum 0.8 + Diesel 2.3 + SQLite (bundled) + Tokio.

Прочитай docs/implementation_plan.md раздел "Структура папок" и "Фаза 0".

Создай новый Rust-проект в папке backend/ (не трогай backend_rust/ — это старый код):

backend/Cargo.toml — зависимости точно как в разделе "Фаза 1 — Зависимости" из docs/implementation_plan.md.

backend/src/main.rs — минимальный Axum сервер:
- читает порт из конфига
- подключает CORS (разрешает origin из конфига)
- монтирует router из api/mod.rs
- запускает tokio::main

backend/src/config.rs — структура Config:
- читает все поля из .env через dotenvy
- поля: database_url, jwt_secret, seafile_url, seafile_token, previews_dir, avatars_dir, server_port, frontend_origin
- паникует при отсутствии обязательных полей

backend/.env.example — все переменные с пустыми значениями и комментариями что куда вписывать.

Без unwrap() на I/O. Без лишних комментариев. Без TODO.
```

---

#### Фаза 1а — База данных: миграция и модели

**Режим: Ask before edits | Effort: High**

📖 Прочитать:
- `docs/database.md` — вся схема, все поля, индексы
- `backend/src/db/mod.rs`

✏️ Создать:
- `backend/migrations/0001_initial/up.sql`
- `backend/migrations/0001_initial/down.sql`
- `backend/src/db/schema.rs`
- `backend/src/db/models.rs`

🚫 Не трогать:
- `backend/src/main.rs`
- `backend/src/config.rs`
- `backend_rust/` — старый код

```
Проект: Errant Fox.
Прочитай docs/database.md — там полная схема БД с объяснением каждого поля.

Создай Diesel-миграцию и модели:

backend/migrations/0001_initial/up.sql:
- 5 таблиц: users, videos, bouts, techniques, comments
- точно по схеме из docs/database.md (все поля, типы, nullable, FK, DEFAULT)
- индексы из раздела "Индексы" в docs/database.md

backend/migrations/0001_initial/down.sql:
- DROP TABLE для каждой таблицы в обратном порядке (сначала зависимые)

backend/src/db/schema.rs:
- Diesel-схема (diesel_migrations table_schema!), соответствует up.sql

backend/src/db/models.rs:
- Rust-структуры для каждой таблицы с derive(Queryable, Insertable, Serialize, Deserialize)
- id: String (UUID), timestamps: chrono::NaiveDateTime, nullable: Option<T>
- отдельные структуры New* для INSERT (без id и created_at)
```

---

#### Фаза 1б — Авторизация

**Режим: Edit automatically | Effort: High**

📖 Прочитать:
- `docs/api.md` раздел "Авторизация" и "Профиль текущего пользователя"
- `backend/src/config.rs`
- `backend/src/db/models.rs`

✏️ Создать / изменить:
- `backend/src/api/auth.rs` (создать)
- `backend/src/middleware/auth.rs` (заполнить)
- `backend/src/api/mod.rs` (добавить маршруты)

🚫 Не трогать:
- `backend/src/db/` — не менять схему
- `backend/migrations/` — не менять миграции

```
Проект: Errant Fox.
Прочитай docs/api.md раздел "Авторизация" и "Профиль текущего пользователя".
Прочитай backend/src/config.rs и backend/src/db/models.rs.

Реализуй авторизацию:

backend/src/api/auth.rs:
- POST /api/auth/login: принимает {username, password}, ищет user в SQLite по username,
  проверяет bcrypt::verify, возвращает {token, user} точно в формате из docs/api.md
- JWT: срок 7 дней, алгоритм HS256, подписывается config.jwt_secret
- при неверном пароле или несуществующем пользователе: 401 с {"error": "Invalid credentials"}

backend/src/middleware/auth.rs:
- Axum FromRequestParts extractor CurrentUser
- читает заголовок Authorization: Bearer <token>
- расшифровывает JWT, достаёт user_id из claims
- загружает пользователя из БД, возвращает User
- при отсутствии/невалидном токене: 401

backend/src/api/mod.rs:
- добавить маршрут POST /api/auth/login
- добавить маршруты GET /api/users/me, PATCH /api/users/me (с middleware CurrentUser)

Ошибки через AppError enum + IntoResponse. Без unwrap() на I/O.
```

---

#### Фаза 2а — REST API: бойцы, техники, видео

**Режим: Edit automatically | Effort: High**

📖 Прочитать:
- `docs/api.md` — разделы "Бойцы", "Техники", "Видео"
- `backend/src/db/models.rs`
- `backend/src/middleware/auth.rs`

✏️ Создать / изменить:
- `backend/src/api/users.rs` (создать)
- `backend/src/api/techniques.rs` (создать)
- `backend/src/api/videos.rs` (создать)
- `backend/src/api/mod.rs` (добавить маршруты)

🚫 Не трогать:
- `backend/src/api/auth.rs`
- `backend/src/middleware/`
- `backend/src/db/`

```
Проект: Errant Fox.
Прочитай docs/api.md разделы "Бойцы", "Техники", "Видео".
Прочитай backend/src/db/models.rs и backend/src/middleware/auth.rs.

Реализуй REST-эндпоинты. Форматы ответов — точно по docs/api.md.

backend/src/api/users.rs:
- GET /api/fighters — список всех пользователей (id, username, display_name, avatar_url, color)
- GET /api/fighters/:id/bouts — все бои бойца, формат из docs/api.md раздел "Бойцы"
- PATCH /api/users/me — обновить display_name и/или password (bcrypt если пароль)
- POST /api/users/me/avatar — multipart, сохранить файл в config.avatars_dir/<user_id>.jpg
- GET /api/users/:id/avatar — отдать файл из avatars_dir
- POST /api/admin/users — создать пользователя (только если CurrentUser.is_admin)
- DELETE /api/admin/users/:id — удалить (только admin, нельзя удалить себя)

backend/src/api/techniques.rs:
- GET /api/techniques — все техники
- POST /api/admin/techniques — добавить (только admin)
- DELETE /api/admin/techniques/:id — удалить (только admin)

backend/src/api/videos.rs:
- GET /api/videos — список с query-параметрами fighter_id, date_from, date_to
  total_score_a/b вычислять как SUM из bouts, не хранить
- GET /api/videos/:id — метаданные + bouts + comments, формат точно по docs/api.md
- PATCH /api/videos/:id — обновить fighter_a_id, fighter_b_id

Все маршруты добавить в backend/src/api/mod.rs.
```

---

#### Фаза 2б — Seafile и превью

**Режим: Edit automatically | Effort: High**

📖 Прочитать:
- `docs/architecture.md` раздел "2. Интеграция с Seafile"
- `docs/api.md` раздел `GET /api/videos/:id/stream` и `GET /api/videos/:id/previews/:frame`
- `backend/src/config.rs`
- `backend/src/db/models.rs`

✏️ Создать / изменить:
- `backend/src/seafile.rs` (создать)
- `backend/src/sync.rs` (создать)
- `backend/src/previews.rs` (создать)
- `backend/src/api/videos.rs` (добавить 2 эндпоинта)
- `backend/src/main.rs` (запустить sync задачу)

🚫 Не трогать:
- `backend/src/api/auth.rs`
- `backend/src/api/users.rs`
- `backend/src/api/techniques.rs`
- `backend/migrations/`

```
Проект: Errant Fox.
Прочитай docs/architecture.md раздел "2. Интеграция с Seafile".
Прочитай backend/src/config.rs и backend/src/db/models.rs.

backend/src/seafile.rs — HTTP-клиент к Seafile REST API:
- struct SeafileClient с полями url, token
- async fn list_folders(&self) → Result<Vec<FolderInfo>> — GET /api2/repos/<repo_id>/dir/
- async fn list_files(&self, folder: &str) → Result<Vec<FileInfo>>
- async fn get_download_url(&self, path: &str) → Result<String> — ссылка на 1 час
- заголовок во всех запросах: Authorization: Token <token>
- используй reqwest

backend/src/sync.rs — фоновая задача:
- async fn run_sync(seafile: Arc<SeafileClient>, db: DbPool, ws_tx: broadcast::Sender<WsEvent>)
- tokio::time::interval(Duration::from_secs(60))
- для каждой папки: парсить дату regex r"(\d{4}-\d{2}-\d{2})"
- если seafile_path не существует в videos — INSERT, отправить WsEvent::NewVideo

backend/src/previews.rs:
- async fn generate_previews(video_id: &str, download_url: &str, previews_dir: &Path)
- запустить FFmpeg: ffmpeg -i <url> -vf "select=not(mod(n\,<step>)),scale=480:-1" -vsync 0 -frames:v 10 <dir>/%d.jpg
- после успеха: UPDATE videos SET preview_count=10 WHERE id=<video_id>

backend/src/api/videos.rs — добавить:
- GET /api/videos/:id/stream → вызвать seafile.get_download_url(), вернуть {stream_url}
- GET /api/videos/:id/previews/:frame → отдать файл из previews_dir/<id>/<frame>.jpg
  если preview_count=0: запустить generate_previews в tokio::spawn, вернуть 202

backend/src/main.rs — добавить tokio::spawn(sync::run_sync(...)) перед serve.
```

---

#### Фаза 2в — Сходы, комментарии, WebSocket

**Режим: Edit automatically | Effort: High**

📖 Прочитать:
- `docs/api.md` — разделы "Сходы", "Комментарии", "WebSocket"
- `backend/src/db/models.rs`
- `backend/src/middleware/auth.rs`

✏️ Создать / изменить:
- `backend/src/api/bouts.rs` (создать)
- `backend/src/api/comments.rs` (создать)
- `backend/src/ws.rs` (создать)
- `backend/src/api/mod.rs` (добавить маршруты)

🚫 Не трогать:
- `backend/src/seafile.rs`
- `backend/src/sync.rs`
- `backend/src/previews.rs`

```
Проект: Errant Fox.
Прочитай docs/api.md разделы "Сходы (Буты)", "Комментарии", "WebSocket".
Прочитай backend/src/db/models.rs.

backend/src/api/bouts.rs:
- POST /api/bouts — создать сход, order_index = MAX(order_index)+1 для этого video_id
- PATCH /api/bouts/:id — обновить поля схода (все опциональны)
- DELETE /api/bouts/:id — удалить
- после POST и PATCH: broadcast WsEvent::UpdateBout всем кто подключён к этому video_id

backend/src/api/comments.rs:
- POST /api/comments — создать комментарий, author_id из CurrentUser
- PATCH /api/comments/:id — изменить text (только свой комментарий), обновить edited_at
- DELETE /api/comments/:id — удалить (свой или если CurrentUser.is_admin)
- после POST: broadcast WsEvent::NewComment

backend/src/ws.rs:
- WsEvent enum: NewComment(Comment), UpdateBout(Bout), NewVideo { id: String, date: String, preview_url: String }
- WsHub: broadcast::Sender<WsEvent>
- handler: GET /ws — upgrade to WebSocket
- первое сообщение от клиента: {token: "..."} — валидировать JWT, записать user_id
- подписка на broadcast, фильтровать: NewComment/UpdateBout — только если клиент 
  открыл это видео (клиент отправляет {watching: video_id} после открытия видео)
- при получении события: сериализовать в JSON, отправить клиенту
```

---

#### Фаза 3 — Frontend: фундамент

**Режим: Edit automatically | Effort: High**

📖 Прочитать:
- `docs/api.md` — все разделы (форматы запросов и ответов)
- `TrueNAS/src/client/src/types.ts` — существующие типы
- `TrueNAS/src/client/package.json` — зависимости

✏️ Создать:
- `frontend/src/lib/api/client.ts`
- `frontend/src/lib/api/types.ts`
- `frontend/src/lib/api/auth.ts`
- `frontend/src/lib/api/videos.ts`
- `frontend/src/lib/api/bouts.ts`
- `frontend/src/lib/api/comments.ts`
- `frontend/src/lib/api/fighters.ts`
- `frontend/src/lib/api/techniques.ts`
- `frontend/src/stores.ts`
- `frontend/src/routes/Auth.svelte`

🚫 Не трогать:
- `TrueNAS/` — только читать для справки, не изменять
- `frontend_noodl/` — мёртвый старый фронтенд
- `backend/`

```
Проект: Errant Fox. Frontend на Svelte 5 + TypeScript + Vite.
Прочитай docs/api.md — все разделы. Это единственный источник правды о формате данных.
Для справки прочитай TrueNAS/src/client/src/types.ts.

frontend/src/lib/api/types.ts:
- TypeScript интерфейсы для КАЖДОГО объекта из docs/api.md
- User, Fighter, FighterBout, Video, VideoFull, Bout, Comment, Technique
- никакого any, все поля typed

frontend/src/lib/api/client.ts:
- async function apiFetch<T>(path, options?): Promise<T>
- добавляет Authorization: Bearer <token из localStorage "ef_token">
- при 401: удалить токен, window.location = '/auth'
- при сетевой ошибке: throw с понятным сообщением на русском

frontend/src/lib/api/auth.ts — login(username, password): Promise<{token, user}>
frontend/src/lib/api/videos.ts — getVideos(filters?), getVideo(id), patchVideo(id, data), getStreamUrl(id)
frontend/src/lib/api/bouts.ts — createBout(data), updateBout(id, data), deleteBout(id)
frontend/src/lib/api/comments.ts — createComment(data), updateComment(id, text), deleteComment(id)
frontend/src/lib/api/fighters.ts — getFighters(), getFighterBouts(id)
frontend/src/lib/api/techniques.ts — getTechniques(), createTechnique(name), deleteTechnique(id)

frontend/src/stores.ts (Svelte stores):
- token: Writable<string | null> (sync с localStorage "ef_token")
- currentUser: Writable<User | null>
- techniques: Writable<Technique[]>
- fighters: Writable<Fighter[]>
- функция initStores(): загружает techniques и fighters при старте

frontend/src/routes/Auth.svelte:
- форма: поля Username и Password, кнопка Sign In
- при submit: вызов login(), сохранить token в store, redirect на /
- при ошибке: показать сообщение под формой
- стиль: тёмно-синий фон, карточка по центру, цвет акцента #DB841F
```

---

#### Фаза 3б — Frontend: роутинг и Header

**Режим: Edit automatically | Effort: Medium**

📖 Прочитать:
- `docs/requirements.md` раздел "5.2. Глобальный Header"
- `frontend/src/stores.ts`
- `TrueNAS/src/client/src/App.svelte`

✏️ Создать / изменить:
- `frontend/src/App.svelte` (создать)
- `frontend/src/lib/ui/Header.svelte` (создать)

🚫 Не трогать:
- `frontend/src/lib/api/`
- `frontend/src/stores.ts`
- `TrueNAS/` — только читать

```
Проект: Errant Fox.
Прочитай docs/requirements.md раздел "5.2. Глобальный Header".
Для справки прочитай TrueNAS/src/client/src/App.svelte.

frontend/src/App.svelte:
- если token не задан → показать Auth.svelte
- иначе: Header + роутер по hash (#/gallery, #/stats, #/player/:id)
- при старте вызвать initStores() из stores.ts

frontend/src/lib/ui/Header.svelte:
- лево: логотип "Errant Fox" → клик → #/gallery
- центр: кнопки [Видео] [Бойцы] — переключение между #/gallery и #/stats
- право: иконка настроек + аватар → dropdown меню:
  - всегда: "Профиль", "Выйти"
  - только если currentUser.is_admin: "Создать бойца", "Техники"
- цвет акцента #DB841F, тёмная тема
```

---

#### Фаза 4 — Галерея

**Режим: Edit automatically | Effort: High**

📖 Прочитать:
- `docs/requirements.md` раздел "5.3. Экран «Видео»"
- `frontend/src/lib/api/types.ts`
- `frontend/src/stores.ts`
- `TrueNAS/src/client/src/lib/asset_browser/VideoTile.svelte`

✏️ Создать:
- `frontend/src/routes/Gallery.svelte`
- `frontend/src/lib/gallery/VideoCard.svelte`
- `frontend/src/lib/gallery/VideoGrid.svelte`
- `frontend/src/lib/gallery/Sidebar.svelte`
- `frontend/src/lib/gallery/TagModal.svelte`

🚫 Не трогать:
- `frontend/src/lib/api/`
- `frontend/src/lib/ui/Header.svelte`
- `TrueNAS/` — только читать

```
Проект: Errant Fox.
Прочитай docs/requirements.md раздел "5.3. Экран «Видео»".
Для справки прочитай TrueNAS/src/client/src/lib/asset_browser/VideoTile.svelte.

frontend/src/lib/gallery/VideoCard.svelte:
- props: video: Video (тип из types.ts)
- превью: <img src="/api/videos/{video.id}/previews/0" loading="lazy">
- hover scrub: mousemove по карточке → frame = Math.floor(x/width * video.preview_count)
  менять src на /api/videos/{id}/previews/{frame}
- если video.is_tagged: аватары, display_name бойцов, счёт "X : Y", дата
- если !video.is_tagged: иконка-плейсхолдер + текст "Заполните данные"
- клик на tagged → emit('open', video.id)
- клик на untagged → emit('tag', video)
- соотношение 16:9, тёмная карточка, Tailwind

frontend/src/lib/gallery/TagModal.svelte:
- props: video: Video
- два select: Боец A и Боец B (options из fighters store, не выбрать одного дважды)
- дата предзаполнена из video.date (readonly)
- кнопка Сохранить → PATCH /api/videos/:id → emit('saved', updatedVideo)
- кнопка Отмена → emit('close')

frontend/src/lib/gallery/Sidebar.svelte:
- чекбоксы бойцов из fighters store, рядом с каждым — количество видео
- два date input: С / По
- при изменении любого фильтра → emit('filter', {fighter_ids: [], date_from, date_to})

frontend/src/routes/Gallery.svelte:
- при mount: загрузить getVideos()
- подписаться на WS событие new_video → добавить карточку
- Sidebar слева, VideoGrid справа
- клик 'open' → router.push('#/player/' + id)
- клик 'tag' → показать TagModal
```

---

#### Фаза 5а — Видео-плеер: базовый

**Режим: Ask before edits | Effort: High**

📖 Прочитать:
- `docs/requirements.md` раздел "5.5. Экран просмотра видео"
- `TrueNAS/src/client/src/lib/player_view/VideoPlayer.svelte` — весь файл
- `TrueNAS/src/client/src/lib/player_view/video-decoder/HybridVideoDecoder.ts`
- `TrueNAS/src/client/src/lib/player_view/video-decoder/Html5VideoDecoder.ts`
- `TrueNAS/src/client/src/lib/player_view/video-decoder/MediabunnyDecoder.ts`
- `TrueNAS/src/client/src/lib/player_view/video-decoder/timecode.ts`

✏️ Создать:
- `frontend/src/lib/player/VideoPlayer.svelte`
- `frontend/src/lib/player/video-decoder/HybridVideoDecoder.ts`
- `frontend/src/lib/player/video-decoder/Html5VideoDecoder.ts`
- `frontend/src/lib/player/video-decoder/MediabunnyDecoder.ts`
- `frontend/src/lib/player/video-decoder/timecode.ts`

🚫 Не трогать:
- `TrueNAS/` — только читать как источник
- `frontend/src/lib/gallery/`
- `frontend/src/lib/api/`

```
Проект: Errant Fox.
Прочитай docs/requirements.md раздел "5.5." (видео-плеер, горячие клавиши).
Прочитай ПОЛНОСТЬЮ TrueNAS/src/client/src/lib/player_view/VideoPlayer.svelte.
Прочитай все файлы из TrueNAS/src/client/src/lib/player_view/video-decoder/.

Перенеси видео-плеер в frontend/src/lib/player/:

Скопируй без изменений:
- video-decoder/HybridVideoDecoder.ts
- video-decoder/Html5VideoDecoder.ts
- video-decoder/MediabunnyDecoder.ts
- video-decoder/timecode.ts

VideoPlayer.svelte — перенести и очистить:
УБРАТЬ: всё импортирующее из clapshot, gRPC, protobuf, drawing board (simple-drawing-board),
  субтитры, совместный просмотр (shared playback state), коллаборативные рисунки.
ОСТАВИТЬ: HTML5 <video>, HybridVideoDecoder, zoom (CSS transform), pan (middlemouse),
  controls bar (play/pause, seek, скорость, громкость, время), горячие клавиши Space/X/Z.

props: src: string (URL видео), duration: number (в секундах)
события: bind:currentTime, bind:paused — для использования родителем

URL видео = props.src (будет передан из GET /api/videos/:id/stream).
```

---

#### Фаза 5б — Таймлайн

**Режим: Edit automatically | Effort: High**

📖 Прочитать:
- `docs/requirements.md` раздел "5.5." (блок "Таймлайн")
- `TrueNAS/src/client/src/lib/player_view/ExchangeTimeline.svelte`
- `frontend/src/lib/api/types.ts`

✏️ Создать:
- `frontend/src/lib/player/Timeline.svelte`

🚫 Не трогать:
- `frontend/src/lib/player/VideoPlayer.svelte`
- `frontend/src/lib/player/video-decoder/`

```
Проект: Errant Fox.
Прочитай docs/requirements.md таймлайн (раздел 5.5).
Для справки прочитай TrueNAS/src/client/src/lib/player_view/ExchangeTimeline.svelte.

frontend/src/lib/player/Timeline.svelte:
props: currentTime: number, duration: number, bouts: Bout[], comments: Comment[]

4 строки на всю ширину экрана:

Строка 1 — маркеры комментариев:
- точка на позиции (comment.timestamp_ms / duration / 1000 * 100)%
- клик → dispatch('seek', comment.timestamp_ms)

Строка 2 — прогресс-бар:
- заполненная часть = currentTime/duration * 100%
- клик/drag → dispatch('seek', timestamp_ms)

Строка 3 — трек сходов:
- цветной прямоугольник для каждого боя
- left = bout.time_start_ms / (duration*1000) * 100%
- width = (bout.time_end_ms - bout.time_start_ms) / (duration*1000) * 100%
- клик → dispatch('loop', {start: bout.time_start_ms, end: bout.time_end_ms})

Строка 4 — controls:
- PLAY/PAUSE, шаг назад, шаг вперёд, LOOP toggle, скорость дропдаун, громкость, время MM:SS
```

---

#### Фаза 5в — Судейская панель

**Режим: Edit automatically | Effort: High**

📖 Прочитать:
- `docs/requirements.md` раздел "5.5." (блок "Панель разметки")
- `TrueNAS/src/client/src/lib/player_view/JudgingPanel.svelte`
- `frontend/src/lib/api/types.ts`
- `frontend/src/stores.ts`

✏️ Создать:
- `frontend/src/lib/player/JudgingPanel.svelte`
- `frontend/src/lib/player/BoutCard.svelte`

🚫 Не трогать:
- `frontend/src/lib/player/VideoPlayer.svelte`
- `frontend/src/lib/player/Timeline.svelte`
- `TrueNAS/` — только читать

```
Проект: Errant Fox.
Прочитай docs/requirements.md раздел "5.5." блок "Панель разметки".
Для справки прочитай TrueNAS/src/client/src/lib/player_view/JudgingPanel.svelte.

frontend/src/lib/player/JudgingPanel.svelte:
props: video: VideoFull, currentTime: number
- два select: Боец A / Боец B (предзаполнены из video.fighter_a/b)
- кнопка START (зелёная): запомнить currentTime → startTime
- кнопка FINISH (красная): POST /api/bouts {video_id, time_start_ms: startTime*1000, time_end_ms: currentTime*1000}
- список BoutCard отсортированный по order_index
- футер: TOTAL SCORE: {sum(score_a)} : {sum(score_b)}
- WS событие update_bout → обновить bout в списке (или удалить если deleted)

frontend/src/lib/player/BoutCard.svelte:
props: bout: Bout, fighters: Fighter[]

Свёрнутый вид: "Сход {order_index} ({formatTime(time_start_ms)} — {formatTime(time_end_ms)})  {score_a} : {score_b}"
Развёрнутый вид (только один одновременно — при открытии другого закрывается):
- для каждого бойца: числовой input очков (+/- кнопки), select техники (из techniques store),
  select зоны поражения (6 вариантов из requirements), radio результата (hit/miss/blocked)
- кнопка Сохранить → PATCH /api/bouts/:id
- кнопка Свернуть → если есть несохранённые изменения — confirm диалог

dirty-флаг: true если форма изменена после последнего сохранения
```

---

#### Фаза 5г — Чат и сборка плеера

**Режим: Edit automatically | Effort: High**

📖 Прочитать:
- `docs/requirements.md` раздел "5.5." (блок "Чат")
- `frontend/src/lib/player/VideoPlayer.svelte`
- `frontend/src/lib/player/JudgingPanel.svelte`
- `frontend/src/lib/player/Timeline.svelte`

✏️ Создать / изменить:
- `frontend/src/lib/player/Chat.svelte` (создать)
- `frontend/src/routes/Player.svelte` (создать)

🚫 Не трогать:
- `frontend/src/lib/player/BoutCard.svelte`
- `frontend/src/lib/player/video-decoder/`

```
Проект: Errant Fox.
Прочитай docs/requirements.md раздел "5.5." блок "Чат".
Прочитай frontend/src/lib/player/VideoPlayer.svelte (чтобы знать его props и events).

frontend/src/lib/player/Chat.svelte:
props: videoId: string, comments: Comment[]
- список: аватар | имя | тайм-код | текст | кнопка "Ответить"
- ответы: смещены на 16px вправо, показывают reply_to текст
- клик на тайм-код → dispatch('seek', comment.timestamp_ms)
- поле ввода: textarea + Enter (без Shift) → POST /api/comments, добавить в список
- если replyTo установлен — добавить reply_to_id в запрос, показать "Ответ для: ..." над полем
- WS событие new_comment → добавить в конец списка

frontend/src/routes/Player.svelte:
- при mount: GET /api/videos/:id, GET /api/videos/:id/stream
- layout: три колонки (JudgingPanel | VideoPlayer | Chat) + Timeline снизу на всю ширину
- JudgingPanel: слева, фиксированная ширина 300px
- VideoPlayer: центр, занимает оставшееся пространство
- Chat: справа, фиксированная ширина 320px
- Timeline: снизу, position:fixed или sticky, высота 80px
- события Timeline 'seek' → VideoPlayer.seekTo(ms)
- события Timeline 'loop' → VideoPlayer.setLoop(start, end)
- события Chat 'seek' → VideoPlayer.seekTo(ms) + пауза
```

---

#### Фаза 6 — Статистика

**Режим: Edit automatically | Effort: High**

📖 Прочитать:
- `docs/requirements.md` раздел "5.4. Экран «Бойцы»"
- `frontend/src/lib/api/types.ts` (тип FighterBout)
- `TrueNAS/src/client/src/lib/player_view/` — FighterProfile если есть

✏️ Создать:
- `frontend/src/routes/Stats.svelte`
- `frontend/src/lib/stats/FighterSidebar.svelte`
- `frontend/src/lib/stats/HistoryTable.svelte`
- `frontend/src/lib/stats/QuickStats.svelte`
- `frontend/src/lib/stats/FrequencyChart.svelte`
- `frontend/src/lib/stats/ResultsChart.svelte`
- `frontend/src/lib/stats/ScoreChart.svelte`
- `frontend/src/lib/stats/BodySilhouette.svelte`

🚫 Не трогать:
- `frontend/src/lib/player/`
- `frontend/src/lib/gallery/`

```
Проект: Errant Fox.
Прочитай ПОЛНОСТЬЮ docs/requirements.md раздел "5.4. Экран «Бойцы»" — там все детали.
Прочитай frontend/src/lib/api/types.ts тип FighterBout.

Главный принцип: один Svelte derived store filteredBouts. 
Все компоненты получают данные только из него. 
Фильтры меняют его — всё обновляется автоматически.

frontend/src/routes/Stats.svelte:
- при выборе бойца: GET /api/fighters/:id/bouts → записать в rawBouts store
- filteredBouts = derived(rawBouts, activeFilters, ...)
- layout: FighterSidebar слева, дашборд справа

HistoryTable.svelte — props: bouts: FighterBout[], dispatch('filter', filters)
- колонки: Дата | Оппонент | Счёт | Мой приём | Мой результат | Приём оппонента | Результат оппонента | →
- каждый столбец: клик на заголовок = сортировка asc/desc
- под заголовком: input/select для фильтрации по этому столбцу
- клик → → dispatch('navigate', video_id)
- изменение любого фильтра → dispatch('filter', activeFilters)

BodySilhouette.svelte — props: bouts: FighterBout[], type: 'dealt' | 'received'
- SVG силуэт человека в T-позе, 6 зон
- для 'dealt': считать hit_zone из my_hit_zone где my_result='hit'
- для 'received': считать hit_zone из opponent_hit_zone где opponent_result='hit'
- интенсивность заливки зоны = count / maxCount * 0.9 + 0.1 (минимум 10% видимости)
- цвет: #DB841F с разной opacity

QuickStats.svelte: 3 блока из filteredBouts через Array groupBy + max count
FrequencyChart.svelte: Chart.js bar, группировка по ISO-неделям
ResultsChart.svelte: Chart.js line, Y +1 победа / -1 поражение, дропдаун фильтра по оппоненту
ScoreChart.svelte: Chart.js line, суммарные очки за тренировку

Установить Chart.js: npm install chart.js
```

---

#### Фаза 7 — Администрирование

**Режим: Edit automatically | Effort: Medium**

📖 Прочитать:
- `docs/requirements.md` раздел "2. Пользователи и права"
- `docs/api.md` разделы "Пользователи — управление [Admin]" и "Техники"
- `frontend/src/lib/ui/Header.svelte`
- `frontend/src/stores.ts`

✏️ Создать / изменить:
- `frontend/src/lib/admin/CreateUserModal.svelte` (создать)
- `frontend/src/lib/admin/TechniquesModal.svelte` (создать)
- `frontend/src/lib/ui/Header.svelte` (добавить вызов модалов)

🚫 Не трогать:
- `frontend/src/lib/player/`
- `frontend/src/lib/gallery/`
- `frontend/src/lib/stats/`

```
Проект: Errant Fox.
Прочитай docs/requirements.md раздел "2. Пользователи и права".
Прочитай frontend/src/lib/ui/Header.svelte и frontend/src/stores.ts.

frontend/src/lib/admin/CreateUserModal.svelte:
- поля: Username (login), Display Name, Password, флаг Admin
- валидация: все поля обязательны кроме Admin
- POST /api/admin/users → добавить пользователя в fighters store
- закрытие: кнопка Отмена или после успеха

frontend/src/lib/admin/TechniquesModal.svelte:
- список текущих техник из techniques store
- input + кнопка Добавить → POST /api/admin/techniques → обновить store
- кнопка удаления рядом с каждой техникой → DELETE /api/admin/techniques/:id → обновить store
- нельзя удалить технику если она используется в бутах (бэкенд вернёт 409 — показать сообщение)

frontend/src/lib/ui/Header.svelte — добавить:
- в dropdown Admin-пунктов: клик "Создать бойца" → показать CreateUserModal
- клик "Техники" → показать TechniquesModal
```

---

#### Фаза 8 — Деплой

**Режим: Ask before edits | Effort: Medium**

📖 Прочитать:
- `backend/src/config.rs`
- `backend/Cargo.toml`
- `frontend/package.json`

✏️ Создать:
- `Dockerfile.backend`
- `Dockerfile.frontend`
- `docker-compose.yml`
- `nginx.conf`
- `deploy.md`

🚫 Не трогать:
- `backend/src/`
- `frontend/src/`
- `docs/`

```
Проект: Errant Fox. Деплой на TrueNAS через Docker.
Прочитай backend/Cargo.toml и frontend/package.json.

Dockerfile.backend — multi-stage:
- stage 1 (builder): FROM rust:1.80-alpine, COPY backend/, RUN cargo build --release
- stage 2: FROM alpine:3.20, COPY бинарник, EXPOSE 8080, CMD ["./errant_fox"]

Dockerfile.frontend — multi-stage:
- stage 1 (builder): FROM node:20-alpine, COPY frontend/, RUN npm ci && npm run build
- stage 2: FROM nginx:alpine, COPY dist/ в /usr/share/nginx/html, COPY nginx.conf

docker-compose.yml:
- сервис backend: build Dockerfile.backend, volumes: ./data:/data, env_file: .env
- сервис frontend: build Dockerfile.frontend, ports: "80:80", depends_on: backend

nginx.conf:
- location / → /usr/share/nginx/html
- location /api/ → http://backend:8080/api/ (proxy_pass)
- location /ws → http://backend:8080/ws (proxy_pass + Upgrade WebSocket заголовки)

deploy.md — инструкция на русском:
1. Установить Docker на TrueNAS
2. Заполнить .env (скопировать из backend/.env.example)
3. docker-compose up -d
4. Как создать первого Admin-пользователя (через sqlite3 или CLI команду)
```

---

### Советы по экономии токенов

**Дробить на части.** Один промпт = один файл или одна функция. "Сделай весь плеер" — дорого и непредсказуемо. "Перенеси только VideoPlayer.svelte" — дёшево и точно.

**Выделять нужный код перед промптом.** Открыть файл в VS Code → выделить нужный кусок → написать промпт. Claude видит выделение и работает только с ним.

**Указывать режим перед вставкой.** Сначала переключить режим (Plan/Edit/Ask), потом вставить промпт.

**"Без объяснений, только код"** — добавить в конец любого промпта. Экономит ~30%.

**После каждой фазы — проверить** что всё компилируется (`cargo build` / `npm run dev`) перед переходом к следующей.