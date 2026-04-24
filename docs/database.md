# Errant Fox — База данных

**Движок:** SQLite (один файл `errant_fox.sqlite` на сервере)
**ORM:** Diesel (Rust)

Всего 5 таблиц. Схема намеренно минимальная.

---

## Таблица `users`

Все пользователи системы. Один человек = один аккаунт. Бойцы — это тоже пользователи.

| Поле | Тип | Описание |
|---|---|---|
| `id` | TEXT PK | UUID, генерируется при создании |
| `username` | TEXT UNIQUE NOT NULL | Логин для входа (латиница, без пробелов) |
| `display_name` | TEXT NOT NULL | Отображаемое имя (любые символы) |
| `password_hash` | TEXT NOT NULL | bcrypt-хеш пароля |
| `is_admin` | BOOLEAN NOT NULL DEFAULT false | Права администратора |
| `avatar_path` | TEXT | Путь к файлу аватара на сервере, nullable |
| `color` | TEXT | HEX-цвет бойца для UI (например `#DB841F`), nullable |
| `created_at` | TIMESTAMP NOT NULL DEFAULT now | Дата регистрации |

**Изменения относительно текущей схемы:**
- `login` → `username` (более понятное название)
- `name` → `display_name`
- `avatar` (base64/url) → `avatar_path` (путь к файлу, файл хранится локально)
- Убрано: `color` остаётся (нужен для визуального различения бойцов)

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
| `duration_ms` | INTEGER | Длительность в миллисекундах, nullable (заполняется при первом открытии) |
| `preview_count` | INTEGER NOT NULL DEFAULT 0 | Сколько превью-кадров сгенерировано |
| `created_at` | TIMESTAMP NOT NULL DEFAULT now | Когда появилось в системе |

**Изменения относительно текущей схемы (`media_files`):**
- Переименована: `media_files` → `videos`
- Добавлено: `seafile_path` — раньше отсутствовало! (видео хранились локально)
- Добавлено: `preview_count` — сколько кадров сгенерировано FFmpeg
- Убрано: `total_score_a`, `total_score_b` — **счёт вычисляется динамически** из `bouts`, не хранится
- Убрано: `user_id` (владелец файла) — у нас нет личных видео
- Убрано: `title` — имена файлов в UI не показываются
- Убрано: `has_thumbnail`, `thumb_sheet_cols/rows`, `recompression_done` — артефакты старой video pipeline
- Убрано: `fps`, `raw_metadata_all` — лишние данные

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
| `hit_zone_a` | TEXT | Зона попадания бойца A, nullable |
| `hit_zone_b` | TEXT | Зона попадания бойца B, nullable |
| `result_a` | TEXT | Результат удара A: `hit` / `miss` / `blocked`, nullable |
| `result_b` | TEXT | Результат удара B: `hit` / `miss` / `blocked`, nullable |

**Допустимые значения `hit_zone`:** `Голова`, `Тело`, `Рука левая`, `Рука правая`, `Нога левая`, `Нога правая`

**Изменения относительно текущей схемы (`hema_bouts`):**
- Переименована: `hema_bouts` → `bouts`
- `video_hash` → `video_id` (более чёткое название)
- `start_time`/`end_time` (FLOAT, секунды) → `time_start_ms`/`time_end_ms` (INTEGER, миллисекунды)
- Добавлено: `order_index` — раньше порядок определялся только через время, что ненадёжно
- Добавлено: `result_a`, `result_b` — **отсутствовали**, хотя были в ТЗ
- Убрано: `start_timecode`/`end_timecode` (TEXT) — дублировали время в другом формате; таймкод вычисляется на фронте

> **Аналитика получённого урона:**
> Статистика "получил удар в голову" для бойца A строится из `hit_zone_b` и `result_b` в тех сходах,
> где этот боец участвует как `fighter_a_id` видео. Отдельно не хранится.

---

## Таблица `techniques`

Глобальный список приёмов/техник, управляется Admin.

| Поле | Тип | Описание |
|---|---|---|
| `id` | INTEGER PK AUTOINCREMENT | |
| `name` | TEXT UNIQUE NOT NULL | Название техники (например «Цорнхау») |

**Изменения относительно текущей схемы (`hema_moves`):**
- Переименована: `hema_moves` → `techniques`
- Без изменений по структуре

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
| `created_at` | TIMESTAMP NOT NULL DEFAULT now | |
| `edited_at` | TIMESTAMP | Когда отредактирован, nullable |

**Изменения относительно текущей схемы:**
- `media_file_id` → `video_id`
- `timecode` (TEXT) → `timestamp_ms` (INTEGER) — числовое значение точнее и удобнее
- Убрано: `drawing` (base64 рисунок) — функция рисования поверх видео не входит в ТЗ
- Убрано: `username_ifnull` — у нас нет анонимных комментариев
- Убрано: `user_id nullable` — все комментарии от залогиненных пользователей

---

## Удалённые таблицы

| Таблица | Причина удаления |
|---|---|
| `messages` | Уведомления через WebSocket, в БД не нужны |
| `media_types` | Не используется в нашей логике |

---

## Индексы

```sql
-- Частые запросы при загрузке галереи
CREATE INDEX idx_videos_date ON videos(date);
CREATE INDEX idx_videos_fighter_a ON videos(fighter_a_id);
CREATE INDEX idx_videos_fighter_b ON videos(fighter_b_id);

-- Загрузка сходов видео
CREATE INDEX idx_bouts_video ON bouts(video_id);

-- Загрузка комментариев видео
CREATE INDEX idx_comments_video ON comments(video_id);

-- Аналитика по бойцу
CREATE INDEX idx_bouts_techniques ON bouts(technique_a_id, technique_b_id);
```

---

## ER-диаграмма (упрощённо)

```
users
  ├── id (PK)
  └── ...

videos
  ├── id (PK)
  ├── fighter_a_id ──→ users.id
  ├── fighter_b_id ──→ users.id
  └── seafile_path

bouts
  ├── id (PK)
  ├── video_id ──────→ videos.id
  ├── technique_a_id → techniques.id
  └── technique_b_id → techniques.id

techniques
  ├── id (PK)
  └── name

comments
  ├── id (PK)
  ├── video_id ──────→ videos.id
  ├── author_id ─────→ users.id
  └── reply_to_id ───→ comments.id
```
