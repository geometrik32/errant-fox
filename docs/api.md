# Errant Fox — REST API Reference

## Общие правила

- Base URL: `http://localhost:8080/api`
- Формат данных: JSON
- Аутентификация: заголовок `Authorization: Bearer <jwt_token>`
- Все эндпоинты кроме `/auth/login` требуют токен
- Эндпоинты с пометкой **[Admin]** — только для администраторов
- Коды ответов: `200 OK`, `201 Created`, `400 Bad Request`, `401 Unauthorized`, `403 Forbidden`, `404 Not Found`

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
    "avatar_url": "/api/users/uuid/avatar"
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
Обновить своё имя и/или пароль.

**Тело запроса** (все поля опциональны):
```json
{
  "display_name": "Иван Иванов",
  "password": "newpassword"
}
```

**Ответ:** обновлённый объект пользователя (как в GET /api/users/me)

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
Список всех пользователей (бойцов). Используется в дропдаунах и статистике.

**Ответ:**
```json
[
  {
    "id": "uuid",
    "username": "ivan",
    "display_name": "Иван",
    "avatar_url": "/api/users/uuid/avatar",
    "color": "#DB841F"
  }
]
```

### GET /api/fighters/:id/bouts
Все бои бойца для аналитики. Полные данные каждого боя.

**Ответ:**
```json
[
  {
    "id": 42,
    "video_id": "uuid",
    "video_date": "2026-04-23",
    "opponent_id": "uuid2",
    "opponent_name": "Пётр",
    "order_index": 1,
    "time_start_ms": 12000,
    "time_end_ms": 25000,
    "my_score": 2,
    "opponent_score": 1,
    "my_technique_id": 3,
    "my_technique_name": "Цорнхау",
    "my_hit_zone": "Голова",
    "my_result": "hit",
    "opponent_technique_id": 5,
    "opponent_technique_name": "Цвергхау",
    "opponent_hit_zone": null,
    "opponent_result": "miss"
  }
]
```

---

## Пользователи — управление [Admin]

### POST /api/admin/users
Создать нового пользователя.

**Тело запроса:**
```json
{
  "username": "petr",
  "display_name": "Пётр",
  "password": "password123",
  "is_admin": false
}
```

**Ответ:** объект созданного пользователя

### DELETE /api/admin/users/:id
Удалить пользователя. Нельзя удалить самого себя.

**Ответ:** `{ "ok": true }`

---

## Техники

### GET /api/techniques
Список всех техник (приёмов).

**Ответ:**
```json
[
  { "id": 1, "name": "Цорнхау" },
  { "id": 2, "name": "Цвергхау" }
]
```

### POST /api/admin/techniques [Admin]
Добавить технику.

**Тело запроса:** `{ "name": "Шильхау" }`

**Ответ:** `{ "id": 3, "name": "Шильхау" }`

### DELETE /api/admin/techniques/:id [Admin]
Удалить технику.

**Ответ:** `{ "ok": true }`

---

## Видео

### GET /api/videos
Список видео с фильтрами.

**Query-параметры** (все опциональны):
- `fighter_id` — показать только видео с этим бойцом (любым из двух)
- `date_from` — дата от (формат `YYYY-MM-DD`)
- `date_to` — дата до

**Ответ:**
```json
[
  {
    "id": "uuid",
    "date": "2026-04-23",
    "fighter_a": { "id": "uuid1", "display_name": "Иван", "avatar_url": "..." },
    "fighter_b": { "id": "uuid2", "display_name": "Пётр", "avatar_url": "..." },
    "total_score_a": 7,
    "total_score_b": 5,
    "is_tagged": true,
    "preview_url": "/api/videos/uuid/previews/0",
    "preview_count": 10
  },
  {
    "id": "uuid2",
    "date": "2026-04-20",
    "fighter_a": null,
    "fighter_b": null,
    "is_tagged": false,
    "preview_url": "/api/videos/uuid2/previews/0",
    "preview_count": 10
  }
]
```

### GET /api/videos/:id
Полные данные видео: метаданные + все сходы + все комментарии.

**Ответ:**
```json
{
  "id": "uuid",
  "date": "2026-04-23",
  "fighter_a": { "id": "uuid1", "display_name": "Иван", "avatar_url": "..." },
  "fighter_b": { "id": "uuid2", "display_name": "Пётр", "avatar_url": "..." },
  "stream_url": "https://seafile.local/files/token/video.mp4",
  "duration_ms": 180000,
  "bouts": [
    {
      "id": 1,
      "order_index": 1,
      "time_start_ms": 12000,
      "time_end_ms": 25000,
      "score_a": 2,
      "score_b": 1,
      "technique_a_id": 3,
      "hit_zone_a": "Голова",
      "result_a": "hit",
      "technique_b_id": null,
      "hit_zone_b": null,
      "result_b": "miss"
    }
  ],
  "comments": [
    {
      "id": 10,
      "author": { "id": "uuid1", "display_name": "Иван", "avatar_url": "..." },
      "timestamp_ms": 14500,
      "text": "Хороший удар!",
      "reply_to_id": null,
      "created_at": "2026-04-23T18:00:00Z"
    }
  ]
}
```

### PATCH /api/videos/:id
Назначить бойцов на неразмеченное видео.

**Тело запроса:**
```json
{ "fighter_a_id": "uuid1", "fighter_b_id": "uuid2" }
```

**Ответ:** обновлённый объект видео (как в GET /api/videos/:id)

### GET /api/videos/:id/stream
Получить временную ссылку для воспроизведения видео из Seafile.

**Ответ:**
```json
{ "stream_url": "https://seafile.local/files/temp-token/video.mp4" }
```

*Ссылка действительна 1 час. Браузер стримит видео напрямую с Seafile.*

### GET /api/videos/:id/previews/:frame
Отдаёт PNG-кадр для превью (frame = 0, 1, 2, ... preview_count-1).

*Кадры генерируются бэкендом через FFmpeg при первом запросе и кешируются.*

---

## Сходы (Буты)

### POST /api/bouts
Создать новый сход.

**Тело запроса:**
```json
{
  "video_id": "uuid",
  "time_start_ms": 12000,
  "time_end_ms": 25000
}
```

**Ответ:** созданный объект схода с `order_index` (присваивается автоматически)

### PATCH /api/bouts/:id
Сохранить данные схода (очки, техники, зоны, результаты).

**Тело запроса** (все поля опциональны):
```json
{
  "time_start_ms": 12000,
  "time_end_ms": 25000,
  "score_a": 2,
  "score_b": 1,
  "technique_a_id": 3,
  "hit_zone_a": "Голова",
  "result_a": "hit",
  "technique_b_id": null,
  "hit_zone_b": null,
  "result_b": "miss"
}
```

**Ответ:** обновлённый объект схода

### DELETE /api/bouts/:id
Удалить сход.

**Ответ:** `{ "ok": true }`

---

## Комментарии

### POST /api/comments
Создать комментарий.

**Тело запроса:**
```json
{
  "video_id": "uuid",
  "timestamp_ms": 14500,
  "text": "Хороший удар!",
  "reply_to_id": null
}
```

**Ответ:** созданный объект комментария

### PATCH /api/comments/:id
Редактировать комментарий (только свой).

**Тело запроса:** `{ "text": "Отредактированный текст" }`

**Ответ:** обновлённый объект комментария

### DELETE /api/comments/:id
Удалить комментарий (свой или Admin).

**Ответ:** `{ "ok": true }`

---

## WebSocket

### WS /ws
Единственное WebSocket-соединение. Подключается после логина.

**Аутентификация:** первое сообщение после connect — `{ "token": "eyJ..." }`.

**События от сервера (JSON):**

| Событие | Данные | Когда |
|---|---|---|
| `new_comment` | объект комментария | другой пользователь оставил комментарий к открытому видео |
| `update_bout` | объект схода | другой пользователь сохранил/удалил сход |
| `new_video` | `{ id, date, preview_url }` | Seafile sync обнаружил новое видео |

*Клиент не отправляет сообщений через WebSocket — только REST запросы.*

---

## Именование TypeScript-типов (фронтенд)

| Тип | Соответствует | Описание |
|---|---|---|
| `User` | GET /api/users/me | Текущий пользователь |
| `Fighter` | GET /api/fighters[] | Боец в списке |
| `FighterBout` | GET /api/fighters/:id/bouts[] | Бой для страницы статистики |
| `Video` | GET /api/videos[] | Карточка галереи (без бутов и комментов) |
| `VideoFull` | GET /api/videos/:id | Видео с бутами и комментариями |
| `VideoShort` | WS new_video | `{ id, date, preview_url }` — только для WS-уведомления |
| `Bout` | bouts[] внутри VideoFull | Сход |
| `Comment` | comments[] внутри VideoFull | Комментарий |
| `Technique` | GET /api/techniques[] | Техника |
