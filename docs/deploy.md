# Деплой Errant Fox на TrueNAS

## Требования

- TrueNAS SCALE (24.04+) с включённым Docker
- SSH-доступ к серверу
- Работающий Seafile на TrueNAS (нужен URL и API-токен)

---

## 1. Включить Docker на TrueNAS SCALE

1. Войдите в веб-интерфейс TrueNAS.
2. **Apps → Settings → Enable Apps** — включите, если ещё не включено.
3. Подключитесь по SSH и проверьте:
   ```bash
   docker --version
   docker compose version
   ```
   Если `docker compose` недоступен, установите плагин:
   ```bash
   apk add docker-cli-compose
   ```

---

## 2. Скопировать проект на сервер

```bash
# Через SSH на TrueNAS
cd /mnt/your-pool/apps        # замените your-pool на имя вашего пула
git clone <url-репозитория> errant-fox
cd errant-fox
```

> Если git не установлен: `apk add git`

---

## 3. Создать .env

Скопируйте пример и заполните все поля:

```bash
cp backend/.env.example .env
nano .env
```

### Обязательные переменные:

| Переменная | Откуда взять | Пример |
|---|---|---|
| `DATABASE_URL` | Путь внутри контейнера (не менять) | `/data/db/errant_fox.db` |
| `JWT_SECRET` | Сгенерировать | `openssl rand -hex 32` |
| `SEAFILE_URL` | Адрес вашего Seafile | `https://seafile.aat-terra.ru` |
| `SEAFILE_TOKEN` | Seafile → Admin → API Token | `abc123def456...` |
| `PREVIEWS_DIR` | Путь внутри контейнера (не менять) | `/data/previews` |
| `AVATARS_DIR` | Путь внутри контейнера (не менять) | `/data/avatars` |
| `SERVER_PORT` | Порт внутри контейнера (не менять) | `8080` |
| `FRONTEND_ORIGIN` | URL вашего приложения | `https://errantfox.aat-terra.ru` |

Сгенерировать `JWT_SECRET`:
```bash
openssl rand -hex 32
```

---

## 4. Собрать и запустить

Сборка из исходников на сервере больше не требуется. Приложение скачивает предсобранные образы из GitHub Container Registry (GHCR).

### Продакшн (с Traefik):

```bash
docker compose pull
docker compose up -d
```

Запуск занимает всего пару секунд, так как скачиваются уже скомпилированные легкие образы.

### Локальное тестирование контейнеров (из исходников):

Если вам нужно собрать образы локально на машине разработчика:

```bash
docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d --build
```

При локальном запуске задайте `FRONTEND_ORIGIN=http://localhost:8081` в `.env`.

### Проверить статус:

```bash
docker compose ps
docker compose logs backend
docker compose logs frontend
```

База данных создаётся и мигрирует **автоматически** при первом старте бэкенда.

Приложение доступно по адресу, указанному в Traefik-лейблах (по умолчанию `https://errantfox.aat-terra.ru`).

---

## 5. Создать первого Admin-пользователя

Endpoint регистрации не предусмотрен — первого пользователя нужно добавить напрямую в базу.

### Шаг 1 — Сгенерировать bcrypt-хеш пароля

На любой машине с Python:
```bash
python3 -c "import bcrypt; print(bcrypt.hashpw(b'your_password', bcrypt.gensalt(12)).decode())"
```
Или онлайн: https://bcrypt.online (cost factor 12).

### Шаг 2 — Вставить пользователя в базу

```bash
docker compose exec backend sh
# Внутри контейнера:
sqlite3 /data/db/errant_fox.db
```

В SQLite-оболочке:
```sql
INSERT INTO users (id, username, display_name, password_hash, is_admin)
VALUES (
  lower(hex(randomblob(16))),
  'admin',
  'Admin',
  '$2b$12$ВСТАВЬТЕ_ХЕШ_СЮДА',
  1
);
.quit
```

После этого войдите в приложение с этим логином и паролем.
Дальнейших пользователей создаёт Admin через интерфейс (Header → CreateUserModal).

---

## 6. Обновление

Поскольку теперь образы собираются автоматически на GitHub:

```bash
# Обновить файл docker-compose.yml (если он менялся)
curl -L https://raw.githubusercontent.com/geometrik32/errant-fox/main/docker-compose.yml -o docker-compose.yml

# Скачать новые версии контейнеров и перезапустить
docker compose pull
docker compose up -d
```

Миграции применяются автоматически при каждом старте.

---

## Резервное копирование

Все данные хранятся в `./data/` рядом с проектом:

```
./data/
  db/         — SQLite база (главный файл — errant_fox.db)
  previews/   — превью-кадры видео
  avatars/    — аватары пользователей
```

Для бэкапа достаточно сохранить папку `./data/` и файл `.env`.

---

## Архитектура Docker

```
Errant Fox/
├── docker-compose.yml        ← Продакшн: backend + frontend (образы GHCR) + Traefik
├── docker-compose.dev.yml    ← Локальный Dev: сборка из исходников, порты наружу
├── backend/
│   ├── Dockerfile            ← Сборка бэкенда (rust:alpine)
│   └── .dockerignore         ← Игнорируемые файлы бэкенда
└── frontend/
    ├── Dockerfile            ← Сборка фронтенда (node:alpine -> nginx:alpine)
    ├── nginx.conf            ← Проксирование /api и /ws внутри контейнера
    └── .dockerignore         ← Игнорируемые файлы фронтенда
```

- **backend**: порт 8080 (внутренний), volume `./data:/data`
- **frontend**: порт 80 (внутренний), nginx раздаёт статику + проксирует `/api` и `/ws`
- **Traefik**: HTTPS через Let's Encrypt, домен `errantfox.aat-terra.ru`

При локальной разработке в Docker (`docker-compose.dev.yml`):
- Убирается зависимость от сети `proxy` (Traefik)
- Порт фронтенда пробрасывается наружу: `8081:80`
- Образы пересобираются локально из папок `backend/` и `frontend/`
- `SEAFILE_URL` и `FRONTEND_ORIGIN` переопределяются для локального окружения

---

## Устранение неполадок

| Симптом | Причина | Решение |
|---|---|---|
| Белый экран | Nginx не получил dist | `docker compose logs frontend` |
| `no such table` | Миграции не прошли | `docker compose logs backend` — смотреть ошибку |
| Seafile sync error в логах | Неверный токен или URL | Проверить `SEAFILE_TOKEN` и `SEAFILE_URL` в `.env` |
| Порт 80 занят | Другое приложение | Использовать `docker-compose.local.yml` с портом 8081 |
| Traefik не подхватывает | Сеть `proxy` не создана | `docker network create proxy` |
| 502 Bad Gateway | Backend не запустился | `docker compose logs backend` |
