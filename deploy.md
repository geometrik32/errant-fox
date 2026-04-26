# Деплой Errant Fox на TrueNAS

## Требования

- TrueNAS SCALE (24.04+) с включённым Docker
- SSH-доступ к серверу
- Работающий Seafile на TrueNAS (нужен URL, токен и ID библиотеки)

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

```bash
cp backend/.env.example .env
nano .env
```

Заполните все поля:

| Переменная | Откуда взять | Пример |
|---|---|---|
| `JWT_SECRET` | Сгенерировать | `openssl rand -hex 32` |
| `SEAFILE_URL` | Адрес вашего Seafile | `http://192.168.1.100:8082` |
| `SEAFILE_TOKEN` | Открыть библиотеку → меню `···` → API Token | `abc123...` |
| `SEAFILE_REPO_ID` | URL при открытии библиотеки в Seafile | `a1b2c3d4-...` |
| `FRONTEND_ORIGIN` | IP вашего TrueNAS | `http://192.168.1.100` |

Переменные `DATABASE_URL`, `PREVIEWS_DIR`, `AVATARS_DIR`, `SERVER_PORT` уже
проставлены правильно в `docker-compose.yml` через `environment:` — менять не нужно.

Сгенерировать JWT_SECRET:
```bash
openssl rand -hex 32
```

---

## 4. Собрать и запустить

```bash
docker compose up -d --build
```

Первый запуск занимает 5–10 минут (сборка Rust и Node).

Проверить статус:
```bash
docker compose ps
docker compose logs backend   # логи бэкенда
docker compose logs frontend  # логи nginx
```

База данных создаётся и мигрирует **автоматически** при первом старте бэкенда.

Приложение доступно по адресу: **http://\<ip-truenas\>**

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
apk add --no-cache sqlite
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
Дальнейших пользователей создаёт Admin через интерфейс (раздел настроек).

---

## 6. Обновление

```bash
git pull
docker compose up -d --build
```

Миграции применяются автоматически при каждом старте.

---

## Резервное копирование

Все данные хранятся в `./data/` рядом с `docker-compose.yml`:

```
./data/
  db/         — SQLite база (главный файл — errant_fox.db)
  previews/   — превью-кадры видео
  avatars/    — аватары пользователей
```

Для бэкапа достаточно сохранить папку `./data/` и файл `.env`.

---

## Устранение неполадок

| Симптом | Причина | Решение |
|---|---|---|
| Белый экран | Nginx не получил dist | `docker compose logs frontend` |
| `no such table` | Миграции не прошли | `docker compose logs backend` — смотреть ошибку |
| Seafile sync error в логах | Неверный токен или ID репозитория | Проверить `SEAFILE_TOKEN` и `SEAFILE_REPO_ID` в `.env` |
| Порт 80 занят | Другое приложение | Поменять `"80:80"` на `"8081:80"` в `docker-compose.yml` |
