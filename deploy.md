# Деплой Errant Fox на TrueNAS

## Требования

- TrueNAS SCALE с включённым Docker (Apps → Advanced Settings)
- Git для клонирования репозитория
- Работающий экземпляр Seafile на TrueNAS

---

## 1. Установить Docker на TrueNAS

TrueNAS SCALE поставляется с Docker в составе Kubernetes (Apps). Для использования `docker-compose` напрямую:

1. В веб-интерфейсе TrueNAS: **System → Shell** (или SSH-соединение).
2. Проверьте наличие Docker:
   ```bash
   docker --version
   docker compose version
   ```
3. Если Docker недоступен — включите Apps в TrueNAS SCALE:
   **Apps → Settings → Enable Apps**, затем перезагрузите.

---

## 2. Скопировать проект на сервер

```bash
# Через SSH на TrueNAS
cd /mnt/your-pool/apps
git clone <url-репозитория> errant-fox
cd errant-fox
```

---

## 3. Заполнить .env

Скопируйте шаблон и заполните переменные:

```bash
cp backend/.env.example .env
nano .env   # или vi .env
```

Обязательные переменные:

| Переменная | Описание | Пример |
|---|---|---|
| `JWT_SECRET` | Случайная строка для подписи токенов | `openssl rand -hex 32` |
| `SEAFILE_URL` | Адрес Seafile | `http://192.168.1.100:8082` |
| `SEAFILE_TOKEN` | API-токен Seafile | Из настроек аккаунта Seafile |
| `SEAFILE_REPO_ID` | ID библиотеки Seafile | Из URL при открытии библиотеки |
| `FRONTEND_ORIGIN` | URL фронтенда | `http://192.168.1.100` |

Сгенерировать `JWT_SECRET`:
```bash
openssl rand -hex 32
```

---

## 4. Запустить контейнеры

```bash
docker compose up -d
```

Проверить статус:
```bash
docker compose ps
docker compose logs -f
```

Приложение будет доступно по адресу: `http://<ip-truenas>`

---

## 5. Создать первого Admin-пользователя

После запуска контейнеров подключитесь к базе данных:

```bash
docker compose exec backend sh
```

Внутри контейнера используйте SQLite:

```bash
# Установить sqlite3 (если нет)
apk add --no-cache sqlite

# Открыть базу
sqlite3 /data/db/errant_fox.db
```

В SQLite-оболочке:

```sql
-- Посмотреть таблицу пользователей
SELECT id, username, role FROM users;

-- Повысить существующего пользователя до admin
UPDATE users SET role = 'admin' WHERE username = 'your_username';

-- Выйти
.quit
```

Если пользователей ещё нет — зарегистрируйтесь через интерфейс (`/register`),
затем повысьте роль через SQLite как показано выше.

---

## 6. Обновление

```bash
git pull
docker compose build --no-cache
docker compose up -d
```

---

## Структура данных

Все данные хранятся в `./data/` на хосте:

```
./data/
  db/         — SQLite база данных
  previews/   — кадры-превью видео
  avatars/    — аватары пользователей
```

Для резервного копирования достаточно сохранить папку `./data/` и файл `.env`.
