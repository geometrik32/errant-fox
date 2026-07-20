# Errant Fox — База данных

**Движок:** SQLite (один файл `errant_fox.db` на сервере)
**ORM:** Diesel (Rust)
**Миграции:** 12 последовательных миграций в `backend/migrations/`

Всего 7 таблиц.

---

## Таблица `users`

Все пользователи системы. Один человек = один аккаунт. Бойцы, администраторы и гостевые учетные записи хранятся в этой таблице.

| Поле | Тип | Описание |
|---|---|---|
| `id` | TEXT PK | UUID (или префикс `guest_` для гостей) |
| `username` | TEXT UNIQUE NOT NULL | Логин для входа (латиница, без пробелов) |
| `display_name` | TEXT NOT NULL | Отображаемое имя (любые символы) |
| `password_hash` | TEXT NOT NULL | bcrypt-хеш пароля |
| `is_admin` | BOOLEAN NOT NULL DEFAULT 0 | Права администратора |
| `avatar_path` | TEXT | Путь к файлу аватара на сервере, nullable |
| `color` | TEXT | HEX-цвет бойца для UI (например `#DB841F`), nullable — генерируется автоматически при первом логине |
| `created_at` | TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP | Дата регистрации |
| `vk_id` | TEXT | ID профиля ВКонтакте (миграция 0006), nullable |
| `role` | TEXT NOT NULL DEFAULT 'fighter' | Роль в системе: `admin`, `fighter`, `guest` (миграция 0009) |

---

## Таблица `videos`

Метаданные видеофайлов из Seafile.

| Поле | Тип | Описание |
|---|---|---|
| `id` | TEXT PK | UUID, генерируется при обнаружении в Seafile |
| `seafile_path` | TEXT UNIQUE NOT NULL | Путь в Seafile: `2026-04-23/fight1.mp4` |
| `fighter_a_id` | TEXT FK→users.id | nullable — не назначен |
| `fighter_b_id` | TEXT FK→users.id | nullable — не назначен |
| `date` | DATE NOT NULL | Дата тренировки, парсится из имени папки Seafile |
| `duration_ms` | INTEGER | Длительность в миллисекундах, nullable (заполняется FFprobe при первом открытии) |
| `preview_count` | INTEGER NOT NULL DEFAULT 0 | Сколько превью-кадров сгенерировано (10) |
| `fps` | REAL | FPS видео, nullable — заполняется через парсинг moov atom при синхронизации (миграция 0005) |
| `created_at` | TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP | Когда появилось в системе |
| `is_ai_labeled` | BOOLEAN NOT NULL DEFAULT 0 | Размечено ли видео ИИ-сервисом (миграция 0010) |
| `is_analyzing` | BOOLEAN NOT NULL DEFAULT 0 | Находится ли видео в процессе фонового ИИ-анализа (миграция 0011) |

> **Вычисляемые значения (не хранятся):**
> - `total_score_a` = `SUM(bouts.score_a)` где `video_id = videos.id`
> - `total_score_b` = `SUM(bouts.score_b)` где `video_id = videos.id`
> - `is_tagged` = `fighter_a_id IS NOT NULL AND fighter_b_id IS NOT NULL`

---

## Таблица `bouts`

Сходы (индивидуальные обмены) внутри видео.

| Поле | Тип | Описание |
|---|---|---|
| `id` | INTEGER PK AUTOINCREMENT | |
| `video_id` | TEXT NOT NULL FK→videos.id | К какому видео относится |
| `order_index` | INTEGER NOT NULL | Порядковый номер в видео (1, 2, 3...) |
| `time_start_ms` | INTEGER NOT NULL | Начало схода в миллисекундах |
| `time_end_ms` | INTEGER NOT NULL | Конец схода в миллисекундах |
| `score_a` | INTEGER NOT NULL DEFAULT 0 | Очки бойца A |
| `score_b` | INTEGER NOT NULL DEFAULT 0 | Очки бойца B |
| `technique_a_id` | INTEGER FK→techniques.id | Приём бойца A, nullable |
| `technique_b_id` | INTEGER FK→techniques.id | Приём бойца B, nullable |
| `hit_zone_a` | TEXT | Зона попадания бойца A в формате `"Зона:x:y"`, nullable |
| `hit_zone_b` | TEXT | Зона попадания бойца B, nullable |
| `result_a` | TEXT | Результат удара A: `hit` / `miss` / `blocked` / `late` / `no_strike` / `disqualification` / `afterblow`, nullable |
| `result_b` | TEXT | Результат удара B: те же значения, nullable |
| `is_ai` | BOOLEAN NOT NULL DEFAULT 0 | Создан ли сход автоматической ИИ-разметкой (миграция 0012) |

**Допустимые значения `hit_zone` (16 зон):**
`Голова`, `Шея`, `Плечо пр.`, `Предплечье пр.`, `Кисть пр.`, `Плечо лев.`, `Предплечье лев.`, `Кисть лев.`, `Тело`, `Таз`, `Бедро пр.`, `Голень пр.`, `Стопа пр.`, `Бедро лев.`, `Голень лев.`, `Стопа лев.`

> Формат зоны: `"Название:x:y"` где x,y — нормализованные координаты (0..1) точки удара на SVG-силуэте. Пример: `"Голова:0.500:0.300"`.

---

## Таблица `bout_history`

История изменений и аудит действий над сходами (миграция 0007).

| Поле | Тип | Описание |
|---|---|---|
| `id` | INTEGER PK AUTOINCREMENT | |
| `bout_id` | INTEGER NOT NULL FK→bouts.id ON DELETE CASCADE | Идентификатор схода |
| `user_id` | TEXT NOT NULL FK→users.id | Автор изменения (UUID пользователя или `"ai"`) |
| `action` | TEXT NOT NULL | Действие (`create`, `update`, `delete`, `ai_generate`) |
| `details` | TEXT | Дополнительные текстовые детали изменения, nullable |
| `created_at` | TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP | Время изменения |

---

## Таблица `techniques`

Глобальный список приёмов/техник, управляется Admin.

| Поле | Тип | Описание |
|---|---|---|
| `id` | INTEGER PK AUTOINCREMENT | |
| `name` | TEXT UNIQUE NOT NULL | Название техники (например «Цорнхау») |
| `description` | TEXT | Описание техники, nullable (добавлено миграцией 0004) |

> При удалении техники ссылки в `bouts.technique_a_id` / `bouts.technique_b_id` обнуляются (`NULL`). Сходы не удаляются.

---

## Таблица `comments`

Комментарии к видео с привязкой к таймкоду.

| Поле | Тип | Описание |
|---|---|---|
| `id` | INTEGER PK AUTOINCREMENT | |
| `video_id` | TEXT NOT NULL FK→videos.id | К какому видео |
| `author_id` | TEXT NOT NULL FK→users.id | Кто написал |
| `timestamp_ms` | INTEGER NOT NULL | Позиция в видео (миллисекунды) |
| `text` | TEXT NOT NULL | Текст комментария |
| `reply_to_id` | INTEGER FK→comments.id | Ответ на другой комментарий, nullable |
| `bout_id` | INTEGER FK→bouts.id | К какому сходу относится, nullable — заполняется автоматически (миграция 0003) |
| `created_at` | TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP | |
| `edited_at` | TIMESTAMP | Когда отредактирован, nullable |
| `guest_nickname` | TEXT | Имя незарегистрированного гостя при отправке по расшаренной ссылке (миграция 0008), nullable |

> `bout_id` определяется бэкендом автоматически при создании комментария: если `timestamp_ms` попадает в диапазон `[time_start_ms, time_end_ms]` какого-либо схода этого видео — записывается его id.

---

## Таблица `comment_reactions`

Реакции пользователей на комментарии (лайки/дизлайки). Добавлена миграцией 0002.

| Поле | Тип | Описание |
|---|---|---|
| `comment_id` | INTEGER NOT NULL FK→comments.id ON DELETE CASCADE | К какому комментарию |
| `user_id` | TEXT NOT NULL FK→users.id ON DELETE CASCADE | Кто поставил |
| `kind` | TEXT NOT NULL CHECK(kind IN ('like', 'dislike')) | Тип реакции |

**PK:** (`comment_id`, `user_id`) — один пользователь может иметь только одну реакцию на комментарий.

---

## История миграций

| Миграция | Содержание |
|---|---|
| `0001_initial` | Создание 5 таблиц: `users`, `videos`, `techniques`, `bouts`, `comments` + индексы |
| `0002_comment_reactions` | Добавление таблицы `comment_reactions` |
| `0003_comment_bout_search` | Добавление столбца `bout_id` в `comments` |
| `0004_technique_description` | Добавление столбца `description` в `techniques` |
| `0005_video_fps` | Добавление столбца `fps` в `videos` |
| `0006_user_vk_id` | Добавление столбца `vk_id` в `users` |
| `0007_bout_history` | Добавление таблицы `bout_history` для отслеживания изменений сходов |
| `0008_guest_comments` | Добавление столбца `guest_nickname` в `comments` |
| `0009_user_role` | Добавление столбца `role` в `users` (значения `admin`, `fighter`, `guest`) |
| `0010_ai_label` | Добавление столбца `is_ai_labeled` в `videos` |
| `0011_video_is_analyzing` | Добавление столбца `is_analyzing` в `videos` |
| `0012_bout_is_ai` | Добавление столбца `is_ai` в `bouts` |

---

## Индексы

```sql
-- Частые запросы при загрузке галереи
CREATE INDEX idx_videos_date      ON videos(date);
CREATE INDEX idx_videos_fighter_a ON videos(fighter_a_id);
CREATE INDEX idx_videos_fighter_b ON videos(fighter_b_id);

-- Загрузка сходов видео
CREATE INDEX idx_bouts_video      ON bouts(video_id);

-- Загрузка комментариев видео
CREATE INDEX idx_comments_video   ON comments(video_id);

-- История сходов
CREATE INDEX idx_bout_history_bout_id ON bout_history(bout_id);

-- Аналитика по бойцу
CREATE INDEX idx_bouts_techniques ON bouts(technique_a_id, technique_b_id);
```

---

## ER-диаграмма

```
users
  ├── id (PK)
  ├── vk_id
  └── role

videos
  ├── id (PK)
  ├── fighter_a_id ──→ users.id
  ├── fighter_b_id ──→ users.id
  ├── seafile_path
  ├── is_ai_labeled
  └── is_analyzing

bouts
  ├── id (PK)
  ├── video_id ──────→ videos.id
  ├── technique_a_id → techniques.id
  ├── technique_b_id → techniques.id
  └── is_ai

bout_history
  ├── id (PK)
  ├── bout_id ───────→ bouts.id (CASCADE)
  ├── user_id ───────→ users.id
  └── action

techniques
  ├── id (PK)
  ├── name
  └── description

comments
  ├── id (PK)
  ├── video_id ──────→ videos.id
  ├── author_id ─────→ users.id
  ├── reply_to_id ───→ comments.id
  ├── bout_id ───────→ bouts.id
  └── guest_nickname

comment_reactions
  ├── comment_id ────→ comments.id (CASCADE)
  ├── user_id ───────→ users.id (CASCADE)
  └── kind
```
