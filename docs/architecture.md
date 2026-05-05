# Errant Fox — Архитектура и распределение функций

> Связанные документы: [API Reference](api.md) · [База данных](database.md) · [Требования](requirements.md)

## Стек технологий

| Слой | Технология | Назначение |
|---|---|---|
| **Frontend** | Svelte 5 + TypeScript + Vite | UI, плеер, интерактивность |
| **Backend** | Rust + Axum 0.8 + SQLite | API, данные, бизнес-логика |
| **База данных** | SQLite (файл `/data/db/errant_fox.db`) | Пользователи, бои, комментарии, реакции |
| **Хранение видео** | Seafile (отдельный сервис) | Сами видеофайлы |
| **Превью кадры** | FFmpeg (запускает бэкенд) | Генерация кадров для scrub-анимации |
| **FPS видео** | moov-парсер (Rust + TypeScript) | Извлечение FPS из MP4 moov atom без скачивания файла |
| **Связь FE ↔ BE** | REST (данные) + WebSocket (live) | Чёткие API-эндпоинты |
| **Графики** | Chart.js (динамический импорт) | Радарные, линейные и bar-чарты в статистике |

Бэкенд и фронтенд — **отдельные папки**, взаимодействуют только через API.

---

## Структура проекта

```
Errant Fox/
├── backend/                    ← Rust сервер (Axum + Diesel)
│   ├── src/
│   │   ├── main.rs             ← Точка входа, CORS, запуск сервера
│   │   ├── config.rs           ← Конфиг из env-переменных
│   │   ├── errors.rs           ← AppError enum + IntoResponse
│   │   ├── state.rs            ← AppState (пул БД, Seafile, WS hub)
│   │   ├── db/
│   │   │   ├── mod.rs          ← init_pool + run_migrations
│   │   │   ├── schema.rs       ← Diesel schema (6 таблиц)
│   │   │   └── models.rs       ← Rust-структуры Queryable/Insertable
│   │   ├── api/
│   │   │   ├── mod.rs          ← Axum router (все маршруты)
│   │   │   ├── auth.rs         ← /api/auth/login, JWT, UserDto
│   │   │   ├── users.rs        ← /api/users/me, /api/fighters, admin users
│   │   │   ├── videos.rs       ← /api/videos (CRUD, stream, previews)
│   │   │   ├── bouts.rs        ← /api/bouts
│   │   │   ├── comments.rs     ← /api/comments + reactions + search
│   │   │   └── techniques.rs   ← /api/techniques + admin
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   └── auth.rs         ← CurrentUser extractor (JWT middleware)
│   │   ├── seafile.rs          ← HTTP-клиент к Seafile API
│   │   ├── sync.rs             ← Фоновая синхронизация Seafile (60 сек)
│   │   ├── previews.rs         ← FFmpeg генерация превью-кадров
│   │   ├── moov.rs             ← Парсинг MP4 moov atom (FPS)
│   │   └── ws.rs               ← WebSocket hub, broadcast 3 событий
│   ├── migrations/             ← 5 Diesel-миграций
│   │   ├── 0001_initial/
│   │   ├── 0002_comment_reactions/
│   │   ├── 0003_comment_bout_search/
│   │   ├── 0004_technique_description/
│   │   └── 0005_video_fps/
│   ├── Cargo.toml
│   └── .env.example
├── frontend/                   ← Svelte 5 + Vite
│   ├── src/
│   │   ├── main.ts
│   │   ├── App.svelte          ← Роутинг, инициализация, темы
│   │   ├── app.css             ← Глобальные стили, CSS-переменные
│   │   ├── stores.ts           ← Svelte stores (token, user, fighters, techniques)
│   │   ├── lib/
│   │   │   ├── api/            ← Клиент к REST API
│   │   │   │   ├── client.ts   ← apiFetch<T> (fetch + Bearer токен)
│   │   │   │   ├── types.ts    ← TypeScript-интерфейсы
│   │   │   │   ├── auth.ts
│   │   │   │   ├── bouts.ts
│   │   │   │   ├── comments.ts
│   │   │   │   ├── fighters.ts
│   │   │   │   ├── techniques.ts
│   │   │   │   └── videos.ts
│   │   │   ├── player/         ← Видео-плеер
│   │   │   │   ├── VideoPlayer.svelte
│   │   │   │   ├── Timeline.svelte
│   │   │   │   ├── JudgingPanel.svelte
│   │   │   │   ├── BoutCard.svelte
│   │   │   │   ├── HitZonePicker.svelte
│   │   │   │   ├── Chat.svelte
│   │   │   │   └── moov.ts     ← Клиентский MP4 FPS-парсер
│   │   │   ├── gallery/        ← Галерея
│   │   │   │   ├── VideoCard.svelte
│   │   │   │   ├── VideoGrid.svelte
│   │   │   │   └── Sidebar.svelte
│   │   │   ├── stats/          ← Статистика
│   │   │   │   ├── FighterSidebar.svelte
│   │   │   │   ├── HistoryTable.svelte
│   │   │   │   ├── QuickStats.svelte   (≈ TopTechniques.svelte)
│   │   │   │   ├── TopTechniques.svelte
│   │   │   │   ├── FrequencyChart.svelte
│   │   │   │   ├── ResultsChart.svelte
│   │   │   │   ├── ScoreChart.svelte
│   │   │   │   ├── RadarChart.svelte
│   │   │   │   ├── BodySilhouette.svelte
│   │   │   │   └── RecentOpponents.svelte
│   │   │   ├── admin/          ← Администрирование
│   │   │   │   ├── CreateUserModal.svelte
│   │   │   │   └── TechniquesModal.svelte
│   │   │   └── ui/             ← Общие компоненты
│   │   │       ├── Header.svelte
│   │   │       ├── ProfileModal.svelte
│   │   │       └── SearchPanel.svelte
│   │   └── routes/             ← Страницы (Auth, Gallery, Stats, Player)
│   ├── package.json
│   └── vite.config.ts
├── infra/                      ← Docker-конфигурация
│   ├── docker-compose.yml
│   ├── docker-compose.local.yml
│   ├── Dockerfile.backend
│   └── Dockerfile.frontend
├── docs/                       ← Документация
└── deploy.md                   ← Инструкция по деплою
```

---

## 1. Авторизация

| Функция | Где | Инструмент |
|---|---|---|
| Форма входа (UI) | Frontend | Svelte компонент [`Auth.svelte`](frontend/src/routes/Auth.svelte) |
| Отправка логина/пароля | Frontend → Backend | POST /api/auth/login |
| Проверка пароля | **Backend** | Rust: bcrypt сравнение хеша |
| Выдача токена доступа | **Backend** | Rust: генерация JWT (срок 7 дней) |
| Хранение токена | Frontend | localStorage (ключ `ef_token`) |
| Проверка токена на каждом запросе | **Backend** | Rust: middleware CurrentUser расшифровывает JWT |
| Выход из системы | Frontend | Удаление токена из localStorage |
| Генерация цвета | **Backend** | Rust: детерминированный цвет из user.id при первом логине |
| Подтверждение цвета при get_me | **Backend** | Если color = null → генерируется и сохраняется |

---

## 2. Интеграция с Seafile

> Seafile — отдельный сервис на том же сервере. Наш бэкенд обращается к нему по HTTP.
> Видеобайты через наш бэкенд **не проходят** — браузер стримит напрямую с Seafile.

| Функция | Где | Инструмент |
|---|---|---|
| Хранение токена Seafile | **Backend** | Конфиг при деплое (env `SEAFILE_URL` + `SEAFILE_TOKEN`) |
| Периодический опрос папок Seafile | **Backend** | Rust: tokio::spawn → reqwest → Seafile REST API (каждые 60 сек) |
| Парсинг даты из имени папки | **Backend** | Rust: regex `(\d{4}-\d{2}-\d{2})` |
| Извлечение FPS из video | **Backend** | Rust: `moov.rs` — парсинг MP4 moov atom через HTTP Range (первые 2 МБ) |
| Добавление новых видео в БД | **Backend** | Rust → SQLite INSERT |
| Уведомление фронта о новом видео | **Backend → Frontend** | WebSocket broadcast `new_video` |
| Получение временной ссылки на видео | **Backend** | Rust: reqwest → Seafile API → отдаёт URL клиенту |
| Стриминг видео | Frontend | Браузер: HTML5 `<video src="...seafile_url...">` |
| Генерация превью-кадров | **Backend** | Rust: скачивает видео частично → FFmpeg → 10 кадров JPG → кеш |
| Раздача превью-кадров | **Backend** | Rust: GET /api/videos/:id/previews/:frame |

---

## 3. Галерея (экран «Видео»)

| Функция | Где | Инструмент |
|---|---|---|
| Запрос списка видео с фильтрами | Frontend → Backend | GET /api/videos?fighter=X&date_from=Y&date_to=Z |
| Фильтрация по бойцам | **Backend** | Rust: SQL WHERE по fighter_a_id / fighter_b_id |
| Фильтрация по диапазону дат | **Backend** | Rust: SQL WHERE по date |
| Рендеринг сетки карточек | Frontend | Svelte компоненты VideoCard + VideoGrid |
| Статичное превью (первый кадр) | Frontend | `<img src="/api/videos/:id/previews/0">` |
| Hover scrub анимация | Frontend | Svelte: mousemove → меняет src превью (кадры 0…N) |
| Маркер «Заполните данные» | Frontend | Svelte: условный рендеринг если fighter_a_id = null |
| Клик на неразмеченное → модал | Frontend | Svelte: TagModal (встроен в VideoCard) |
| Выбор бойцов A/B в модале | Frontend | Svelte: дропдауны со списком из /api/fighters |
| Сохранение бойцов A/B | Frontend → Backend | PATCH /api/videos/:id |
| Переход в видео-плеер | Frontend | Svelte Router (hash-based) |
| Поиск по комментариям | Frontend → Backend | GET /api/comments/search?q=... → SearchPanel |

---

## 4. Видео-плеер

### 4.1. Воспроизведение

| Функция | Где | Инструмент |
|---|---|---|
| Загрузка метаданных видео (бойцы, буты, комменты) | Frontend → Backend | GET /api/videos/:id |
| Стриминг видео | **Браузер напрямую с Seafile** | HTML5 `<video>` |
| Play / Pause | Frontend | `videoElement.play()` / `.pause()` |
| Клик на видео = play/pause | Frontend | Svelte: click listener |
| Seek по клику на прогресс-бар | Frontend | `videoElement.currentTime = t` |
| Цифровой зум (колесо мыши) | Frontend | CSS `transform: scale()` с origin в точке курсора |
| Покадровый шаг вперёд (X) | Frontend | HybridVideoDecoder → WebCodecs |
| Покадровый шаг назад (Z) | Frontend | HybridVideoDecoder → WebCodecs |
| Пауза (Space) | Frontend | Svelte: keydown listener |
| Контроль скорости (0.1×–2.0×) | Frontend | `videoElement.playbackRate` |
| Громкость | Frontend | `videoElement.volume` |
| Loop (зацикливание диапазона) | Frontend | Svelte: watchdog на timeupdate → seek к start |

### 4.2. Таймлайн (нижняя панель)

| Функция | Где | Инструмент |
|---|---|---|
| Основной прогресс-бар | Frontend | Svelte: `currentTime / duration * 100%` |
| Маркеры комментариев (точки) | Frontend | Svelte: SVG/div по `timestamp_ms / duration` |
| Трек сходов (цветные прямоугольники) | Frontend | Svelte: `time_start_ms / duration` → ширина и позиция |
| Клик на сегмент схода → loop | Frontend | Svelte: устанавливает loopStart/loopEnd + включает loop |
| Клик на маркер комментария → seek | Frontend | `videoElement.currentTime = comment.timestamp_ms / 1000` |

---

## 5. Разметка сходов (Судейская панель)

| Функция | Где | Инструмент |
|---|---|---|
| Дропдауны бойцов A/B | Frontend | Svelte: предзаполнены из метаданных видео |
| Нажать START → зафиксировать начало | Frontend | Svelte: записывает `currentTime` |
| Нажать FINISH → отправить сход | Frontend → Backend | POST /api/bouts (video_id, time_start_ms, time_end_ms) |
| Сохранение схода в БД | **Backend** | Rust → SQLite |
| Broadcast схода всем зрителям | **Backend** | WebSocket: рассылка `update_bout` |
| Список карточек сходов | Frontend | Svelte: отсортировано по order_index |
| Раскрытие/закрытие карточки | Frontend | Svelte: локальный стейт (аккордеон) |
| Поля карточки: очки, техника, зона, результат | Frontend | Svelte: форма с дропдаунами + HitZonePicker |
| Выбор зоны (HitZonePicker) | Frontend | Svelte: SVG-силуэт с 16 кликабельными зонами |
| Список техник для дропдауна | Frontend (кеш) | Загружается один раз: GET /api/techniques |
| Зоны поражения (enum) | Frontend | Константа: 16 значений (HitZonePicker) |
| Сохранить карточку | Frontend → Backend | PATCH /api/bouts/:id |
| Удалить сход | Frontend → Backend | DELETE /api/bouts/:id |
| TOTAL SCORE в футере | Frontend | Svelte: `sum(score_a) : sum(score_b)` из загруженных бутов |
| Live-обновление | Frontend ← Backend | WS событие `update_bout` → обновить/удалить bout в списке |

---

## 6. Чат и комментарии

| Функция | Где | Инструмент |
|---|---|---|
| Загрузка комментариев при открытии видео | Frontend → Backend | В составе GET /api/videos/:id |
| Отображение треда комментариев | Frontend | Svelte: рекурсивный компонент (reply_to_id) |
| Поле ввода + Enter = отправить | Frontend | Svelte: форма |
| Привязка к текущей позиции видео | Frontend | Svelte: берёт `currentTime` в момент отправки |
| Авто-привязка к сходу | **Backend** | Определяет bout_id по timestamp_ms |
| Сохранение комментария в БД | **Backend** | Rust → SQLite |
| Real-time рассылка другим зрителям | **Backend** | WebSocket broadcast `new_comment` |
| Получение нового комментария live | Frontend | WebSocket listener → добавляет в список |
| Клик на тайм-код → seek + пауза | Frontend | `currentTime = ms / 1000` + `pause()` |
| Ответить на комментарий | Frontend → Backend | POST /api/comments (с reply_to_id) |
| Лайк / дизлайк комментария | Frontend → Backend | POST /api/comments/:id/react |
| Убрать реакцию | Frontend → Backend | DELETE /api/comments/:id/react |
| Поиск по комментариям | Frontend → Backend | GET /api/comments/search?q=... |
| Все видят все комментарии | **Backend** | Rust: без фильтрации по пользователю |

---

## 7. Статистика (экран «Бойцы»)

> Данные загружаются с бэкенда один раз. Все вычисления, фильтрация, графики — **на фронтенде**.

| Функция | Где | Инструмент |
|---|---|---|
| Список бойцов в сайдбаре | Frontend (кеш) | GET /api/fighters |
| Загрузка всех боёв бойца | Frontend → Backend | GET /api/fighters/:id/bouts |
| Таблица «История боёв» | Frontend | Svelte: HistoryTable с сортировкой/фильтрацией |
| Фильтрация по столбцам таблицы | Frontend | Svelte: реактивный filtered-массив |
| Блок «Топ техник» | Frontend | Svelte: TopTechniques — groupBy technique + max count |
| Квик-блоки: чаще всего использую/промахиваюсь/получаю | Frontend | Svelte: QuickStats — computed из filtered-массива |
| Радарная диаграмма результатов | Frontend | Chart.js: RadarChart — 7 осей (hit/afterblow/late/disqualification/no_strike/miss/blocked) |
| График: частота поединков (X=недели, Y=кол-во) | Frontend | Chart.js: FrequencyChart |
| График: динамика результатов (победа/поражение) | Frontend | Chart.js: ResultsChart + фильтр по оппоненту |
| Процент побед/поражений | Frontend | Svelte: `wins / total * 100` |
| График: прогресс по баллам | Frontend | Chart.js: ScoreChart |
| Силуэт: нанесённый урон | Frontend | SVG (BodySilhouette) с динамической заливкой 6 зон |
| Силуэт: полученный урон | Frontend | SVG — данные бойца B в бутах где A участвовал |
| Последние оппоненты | Frontend | Svelte: RecentOpponents — список с win/loss |
| Дата первого зафиксированного боя | Frontend | Svelte: min(date) из массива боёв |
| Клик → → переход к видео | Frontend | Svelte Router hash `#/player/video_id?t=timestamp_ms` |

---

## 8. Управление пользователями

| Функция | Где | Инструмент |
|---|---|---|
| Создать пользователя (Admin) | Frontend → Backend | POST /api/admin/users |
| Изменить пользователя (Admin) | Frontend → Backend | PATCH /api/admin/users/:id |
| Загрузить аватар пользователю (Admin) | Frontend → Backend | POST /api/admin/users/:id/avatar |
| Хеширование пароля | **Backend** | Rust: bcrypt |
| Просмотр профиля | Frontend | GET /api/users/me |
| Редактирование профиля | Frontend → Backend | PATCH /api/users/me (username, display_name, password, color) |
| Загрузка/смена аватара | Frontend → Backend | POST /api/users/me/avatar |
| Проверка прав Admin | **Backend** | Rust: middleware читает is_admin из JWT |
| Удалить пользователя (Admin) | Frontend → Backend | DELETE /api/admin/users/:id |

---

## 9. Управление техниками (Admin)

| Функция | Где | Инструмент |
|---|---|---|
| Список всех техник | Frontend → Backend | GET /api/techniques |
| Добавить технику | Frontend → Backend | POST /api/admin/techniques (name + description) |
| Изменить технику | Frontend → Backend | PATCH /api/admin/techniques/:id |
| Удалить технику | Frontend → Backend | DELETE /api/admin/techniques/:id |
| Хранение списка | **Backend** | SQLite: таблица techniques |

---

## Схема взаимодействия

```
[ Браузер ]                    [ Backend (Rust) ]           [ Seafile ]
    │                                  │                         │
    │── POST /api/auth/login ─────────►│                         │
    │◄── JWT токен ───────────────────│                         │
    │                                  │                         │
    │── GET /api/videos ──────────────►│── HTTP GET (список) ──►│
    │◄── список видео ────────────────│◄── папки/файлы ─────────│
    │                                  │                         │
    │── GET /api/videos/:id/previews ─►│── FFmpeg (нарезка) ───►│
    │◄── превью кадры (JPG) ──────────│    (один раз, кеш)      │
    │                                  │                         │
    │── GET /api/videos/:id ──────────►│── GET seafile_url ────►│
    │◄── метаданные + seafile_url ────│◄── временная ссылка ────│
    │                                  │                         │
    │── видео-поток ──────────────────────────────────────────►│
    │◄── байты видео ────────────────────────────────────────────│
    │   (браузер стримит напрямую)     │                         │
    │                                  │                         │
    │── WS: connect ──────────────────►│                         │
    │── WS: {"token":"..."} ──────────►│── валидация JWT         │
    │── WS: {"watching":"video-id"} ──►│── фильтр событий        │
    │◄── WS: new_comment ───────────────│── SQLite INSERT         │
    │◄── WS: update_bout ──────────────│── SQLite UPDATE/DELETE  │
    │◄── WS: new_video ────────────────│── Seafile sync found    │
```

---

## WebSocket — только live-события

WebSocket используется **только** там, где нужно мгновенное обновление у нескольких пользователей:

| Событие | Направление | Фильтрация |
|---|---|---|
| Новый комментарий (`new_comment`) | Backend → клиенты, watching video_id | По video_id |
| Изменение/удаление схода (`update_bout`) | Backend → клиенты, watching video_id | По video_id |
| Новое видео появилось в Seafile (`new_video`) | Backend → все подключённые | Без фильтрации |

Клиент отправляет `{"watching": "<video_id>"}` при открытии видео и `{"watching": null}` при закрытии. События `new_comment` и `update_bout` доставляются только тем, кто watching нужное видео.

Всё остальное — REST.

---

## Что хранится где

| Данные | Где хранится |
|---|---|
| Видеофайлы | Seafile |
| Превью-кадры (JPG) | Локальная папка на сервере (`/data/previews/`) |
| Аватары пользователей | Локальная папка на сервере (`/data/avatars/`) |
| Пользователи, бойцы, права | SQLite: `users` |
| Видео-метаданные, seafile_path, FPS | SQLite: `videos` |
| Буты (сходы) | SQLite: `bouts` |
| Техники | SQLite: `techniques` |
| Комментарии | SQLite: `comments` |
| Реакции на комментарии | SQLite: `comment_reactions` |
| JWT-токены | Только у клиента (localStorage, ключ `ef_token`) |
