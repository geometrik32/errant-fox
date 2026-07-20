# Errant Fox — REST API Reference

## Общие правила

- Base URL: `http://localhost:8080/api`
- Формат данных: JSON
- Аутентификация: заголовок `Authorization: Bearer <jwt_token>` или Query-параметр `token` для публичных/share-эндпоинтов
- Все приватные эндпоинты кроме `/auth/login` и `/shared/*` требуют авторизацию
- Эндпоинты с пометкой **[Admin]** — только для администраторов
- Коды ответов: `200 OK`, `201 Created`, `202 Accepted`, `400 Bad Request`, `401 Unauthorized`, `403 Forbidden`, `404 Not Found`, `500 Internal Server Error`

---

## Авторизация

### POST /api/auth/login
Вход в систему. Токен не требуется.

**Тело запроса:**
```json
{ "username": "ivan", "password": "secret" }
```

**Ответ:**
```json
{
  "token": "eyJ...",
  "user": {
    "id": "uuid",
    "username": "ivan",
    "display_name": "Иван",
    "is_admin": false,
    "avatar_url": "/api/users/uuid/avatar",
    "color": "#DB841F"
  }
}
```

---

## Профиль текущего пользователя

### GET /api/users/me
Данные текущего пользователя.

**Ответ:**
```json
{
  "id": "uuid",
  "username": "ivan",
  "display_name": "Иван",
  "is_admin": false,
  "avatar_url": "/api/users/uuid/avatar",
  "color": "#DB841F"
}
```

### PATCH /api/users/me
Обновить свои данные (имя, пароль, цвет).

**Тело запроса** (все поля опциональны):
```json
{
  "username": "ivan_new",
  "display_name": "Иван Иванов",
  "password": "newpassword",
  "color": "#FF8800"
}
```

**Ответ:** обновлённый объект пользователя.

### POST /api/users/me/avatar
Загрузить аватар. Формат: `multipart/form-data`, поле `avatar` (PNG/JPG, макс. 2 МБ).

**Ответ:**
```json
{ "avatar_url": "/api/users/uuid/avatar" }
```

### GET /api/users/:id/avatar
Отдаёт файл изображения аватара.

---

## Бойцы

### GET /api/fighters
Список всех пользователей (бойцов).

**Ответ:**
```json
[
  {
    "id": "uuid",
    "username": "ivan",
    "display_name": "Иван",
    "avatar_url": "/api/users/uuid/avatar",
    "color": "#DB841F",
    "is_admin": false
  }
]
```

### GET /api/fighters/:id/bouts
Получить все бои с участием бойца для статистики.

---

## Видео

### GET /api/videos
Список всех видеозаписей галереи.

**Query-параметры:**
- `fighter` (опционально) — фильтр по ID бойца
- `date_from` (опционально) — дата `YYYY-MM-DD`
- `date_to` (опционально) — дата `YYYY-MM-DD`

### GET /api/videos/:id
Полные данные о видео (включая списки сходов, комментариев и статус ИИ-анализа).

**Ответ:**
```json
{
  "id": "uuid",
  "seafile_path": "2026-04-23/fight1.mp4",
  "fighter_a": { "id": "uuid1", "display_name": "Иван", "avatar_url": "...", "color": "#..." },
  "fighter_b": { "id": "uuid2", "display_name": "Пётр", "avatar_url": "...", "color": "#..." },
  "date": "2026-04-23",
  "duration_ms": 120000,
  "fps": 59.94,
  "total_score_a": 5,
  "total_score_b": 3,
  "is_ai_labeled": true,
  "is_analyzing": false,
  "bouts": [...],
  "comments": [...]
}
```

### PATCH /api/videos/:id
Назначить бойцов A/B на неразмеченное видео.

**Тело запроса:**
```json
{ "fighter_a_id": "uuid1", "fighter_b_id": "uuid2" }
```

### GET /api/videos/:id/stream
Получить ссылку прямого воспроизведения из Seafile.

### GET /api/videos/:id/previews/:frame
Получить статичный JPEG-кадр (frame = 0..9).

---

## ИИ-разметка сходов (Whisper AI)

### POST /api/videos/:id/ai-label **[Admin]**
Запустить асинхронный фоновый процесс ИИ-анализа видео и автогенерации сходов.

**Ответ:** `202 Accepted`
```json
{
  "status": "processing",
  "video_id": "uuid"
}
```

### POST /api/videos/:id/cancel-analysis **[Admin]**
Отменить текущий или находящийся в очереди процесс ИИ-анализа видео.

**Ответ:** `200 OK`
```json
{
  "status": "cancelled"
}
```

### GET /api/videos/:id/transcript **[Admin]**
Запросить просмотрщик расшифровки ИИ (Exchange Inspector HTML).

---

## Публичные ссылки и шаринг (Shared Access)

### POST /api/videos/:id/share
Сгенерировать публичную Share-ссылку (JWT токен) на видео или конкретный сход.

**Тело запроса** (опционально):
```json
{
  "bout_id": 15
}
```

**Ответ:**
```json
{
  "token": "eyJhbGciOi..."
}
```

### GET /api/shared/videos/:id?token=...
Получить данные видео для гостевого просмотра без авторизации.

### POST /api/shared/videos/:id/comments?token=...
Добавить гостевой комментарий по расшаренной ссылке.

**Тело запроса:**
```json
{
  "timestamp_ms": 15400,
  "text": "Отличная атака!",
  "guest_nickname": "Анатолий"
}
```

### GET /api/shared/videos/:id/download?token=...
Скачать полное расшаренное видео.

### GET /api/shared/bouts/:id/download?token=...
Скачать файл нарезки конкретного схода.

---

## Администрирование и Синхронизация

### GET /api/admin/videos/sync-check **[Admin]**
Проверить список файлов в Seafile и выявить удаленные из облака записи.

### POST /api/admin/videos/sync-clean **[Admin]**
Удалить из SQLite записи о видеофайлах, которые больше не существуют в Seafile.

### POST /api/admin/videos/import **[Admin]**
Запустить принудительную синхронизацию папок тренировок Seafile.

---

## Сходы (Буты)

### POST /api/bouts
Создать новый сход вручную.

**Тело запроса:**
```json
{
  "video_id": "uuid",
  "time_start_ms": 12000,
  "time_end_ms": 25000
}
```

### PATCH /api/bouts/:id
Сохранить детали схода (очки, техники, зоны, результаты).

### DELETE /api/bouts/:id
Удалить сход.

---

## Комментарии и Реакции

### POST /api/comments
Создать новый комментарий к видео.

### PATCH /api/comments/:id
Отредактировать текст своего комментария.

### DELETE /api/comments/:id
Удалить комментарий.

### POST /api/comments/:id/react
Поставить или изменить реакцию (`like` / `dislike`).

### DELETE /api/comments/:id/react
Убрать свою реакцию.

### GET /api/comments/search
Полнотекстовый поиск по комментариям.

---

## WebSocket API

### WS /ws
Единственное WebSocket-соединение с автоматической реподключением.

**Сообщения от клиентов:**
- `{ "token": "eyJ..." }` — авторизация при подключении
- `{ "watching": "video-uuid" }` — подписка на обновления видео (`new_comment`, `update_bout`)

**События сервера:**

| Событие | Данные | Описание |
|---|---|---|
| `new_comment` | `WsComment` | Новый комментарий к открытому видео |
| `update_bout` | `WsBout` | Создание/изменение/удаление схода |
| `new_video` | `{ id, date, preview_url }` | Обнаружено новое видео при синхронизации Seafile |
| `UpdateVideoAiLabeled` | `{ video_id, is_ai_labeled, is_analyzing }` | Изменение статуса ИИ-анализа видео |
| `UpdateVideoScore` | `{ video_id, total_score_a, total_score_b }` | Обновление суммарного счета видео |
