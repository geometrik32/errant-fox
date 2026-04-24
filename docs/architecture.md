# Errant Fox — Архитектура и распределение функций

> Связанные документы: [API Reference](api.md) · [База данных](database.md) · [Требования](requirements.md)

## Стек технологий

| Слой | Технология | Назначение |
|---|---|---|
| **Frontend** | Svelte 5 + TypeScript + Vite | UI, плеер, интерактивность |
| **Backend** | Rust + Axum + SQLite | API, данные, бизнес-логика |
| **База данных** | SQLite (файл на диске) | Пользователи, бои, комментарии |
| **Хранение видео** | Seafile (отдельный сервис) | Сами видеофайлы |
| **Превью кадры** | FFmpeg (запускает бэкенд) | Генерация кадров для scrub-анимации |
| **Связь FE ↔ BE** | REST (данные) + WebSocket (live) | Чёткие API-эндпоинты |

Бэкенд и фронтенд — **отдельные папки**, взаимодействуют только через API.

---

## 1. Авторизация

| Функция | Где | Инструмент |
|---|---|---|
| Форма входа (UI) | Frontend | Svelte компонент |
| Отправка логина/пароля | Frontend → Backend | POST /api/auth/login |
| Проверка пароля | **Backend** | Rust: bcrypt сравнение хеша |
| Выдача токена доступа | **Backend** | Rust: генерация JWT (срок 7 дней) |
| Хранение токена | Frontend | localStorage браузера |
| Проверка токена на каждом запросе | **Backend** | Rust: middleware расшифровывает JWT |
| Выход из системы | Frontend | Удаление токена из localStorage |

---

## 2. Интеграция с Seafile

> Seafile — отдельный сервис на том же сервере. Наш бэкенд обращается к нему по HTTP.
> Видеобайты через наш бэкенд **не проходят** — браузер стримит напрямую с Seafile.

| Функция | Где | Инструмент |
|---|---|---|
| Хранение токена Seafile | **Backend** | Конфиг при деплое (env-переменная) |
| Периодический опрос папок Seafile | **Backend** | Rust: reqwest → Seafile REST API |
| Парсинг даты из имени папки | **Backend** | Rust: regex (формат `YYYY-MM-DD`) |
| Добавление новых видео в БД | **Backend** | Rust → SQLite |
| Уведомление фронта о новом видео | **Backend → Frontend** | WebSocket broadcast |
| Получение временной ссылки на видео | **Backend** | Rust: reqwest → Seafile API → отдаёт URL клиенту |
| Стриминг видео | Frontend | Браузер: HTML5 `<video src="...seafile_url...">` |
| Генерация превью-кадров | **Backend** | Rust: скачивает видео частично → запускает FFmpeg → нарезает кадры → кеширует на диске |
| Раздача превью-кадров | **Backend** | Rust: GET /api/videos/:id/previews |

---

## 3. Галерея (экран «Видео»)

| Функция | Где | Инструмент |
|---|---|---|
| Запрос списка видео с фильтрами | Frontend → Backend | GET /api/videos?fighter=X&date_from=Y&date_to=Z |
| Фильтрация по бойцам | **Backend** | Rust: SQL WHERE по fighter_a_id / fighter_b_id |
| Фильтрация по диапазону дат | **Backend** | Rust: SQL WHERE по date |
| Рендеринг сетки карточек | Frontend | Svelte компонент VideoCard |
| Статичное превью (первый кадр) | Frontend | `<img src="/api/videos/:id/previews/0">` |
| Hover scrub анимация | Frontend | Svelte: mousemove → меняет src превью (кадры 0…N) |
| Маркер «Заполните данные» | Frontend | Svelte: условный рендеринг если fighter_a_id = null |
| Клик на неразмеченное → модал | Frontend | Svelte: модальный компонент |
| Выбор бойцов A/B в модале | Frontend | Svelte: дропдауны со списком из /api/fighters |
| Сохранение бойцов A/B | Frontend → Backend | PATCH /api/videos/:id |
| Переход в видео-плеер | Frontend | Svelte Router |

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
| Цифровой зум (колесо мыши) | Frontend | CSS `transform: scale()` с origin в точке курсора, индикатор в углу |
| Покадровый шаг вперёд (X) | Frontend | HybridVideoDecoder → WebCodecs (Mediabunny) |
| Покадровый шаг назад (Z) | Frontend | HybridVideoDecoder → WebCodecs (Mediabunny) |
| Пауза (Space) | Frontend | Svelte: keydown listener |
| Контроль скорости (0.1×–2.0×) | Frontend | `videoElement.playbackRate` |
| Громкость | Frontend | `videoElement.volume` |
| Отображение текущего времени | Frontend | Svelte: `timeupdate` event |
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
| Broadcast нового схода всем зрителям | **Backend** | WebSocket: рассылка всем кто открыл это видео |
| Список карточек сходов | Frontend | Svelte: отсортировано по order_index |
| Раскрытие/закрытие карточки | Frontend | Svelte: локальный стейт |
| Несохранённые данные → диалог | Frontend | Svelte: dirty-флаг + confirm dialog |
| Поля карточки: очки, техника, зона, результат | Frontend | Svelte: форма с дропдаунами |
| Список техник для дропдауна | Frontend (кеш) | Загружается один раз: GET /api/techniques |
| Зоны поражения (enum) | Frontend | Константа: 6 фиксированных значений |
| Сохранить карточку | Frontend → Backend | PATCH /api/bouts/:id |
| Удалить сход | Frontend → Backend | DELETE /api/bouts/:id |
| TOTAL SCORE в футере | Frontend | Svelte: `sum(score_a) : sum(score_b)` из загруженных бутов |

---

## 6. Чат и комментарии

| Функция | Где | Инструмент |
|---|---|---|
| Загрузка комментариев при открытии видео | Frontend → Backend | GET /api/videos/:id/comments |
| Отображение треда комментариев | Frontend | Svelte: рекурсивный компонент (reply_to_id) |
| Поле ввода + Enter = отправить | Frontend | Svelte: форма |
| Привязка к текущей позиции видео | Frontend | Svelte: берёт `currentTime` в момент отправки |
| Сохранение комментария в БД | **Backend** | Rust → SQLite |
| Real-time рассылка другим зрителям | **Backend** | WebSocket broadcast |
| Получение нового комментария live | Frontend | WebSocket listener → добавляет в список |
| Клик на тайм-код → seek + пауза | Frontend | `currentTime = ms / 1000` + `pause()` |
| Ответить на комментарий | Frontend → Backend | POST /api/comments (с reply_to_id) |
| Все видят все комментарии | **Backend** | Rust: без фильтрации по пользователю |

---

## 7. Статистика (экран «Бойцы»)

> Данные загружаются с бэкенда один раз. Все вычисления, фильтрация, графики — **на фронтенде**.
> Это позволяет мгновенно реагировать на фильтры без запросов к серверу.

| Функция | Где | Инструмент |
|---|---|---|
| Список бойцов в сайдбаре | Frontend (кеш) | GET /api/fighters |
| Загрузка всех боёв бойца | Frontend → Backend | GET /api/fighters/:id/bouts (полный список с деталями) |
| Таблица «История боёв» | Frontend | Svelte: таблица с сортировкой/фильтрацией |
| Фильтрация по столбцам таблицы | Frontend | Svelte: реактивный filtered-массив |
| График 1: частота поединков (X=недели, Y=кол-во) | Frontend | Chart.js — считает из filtered-массива |
| График 2: динамика результатов (победа/поражение) | Frontend | Chart.js — победитель = у кого сумма очков больше |
| Фильтр графика 2 по оппоненту | Frontend | Svelte: дропдаун → фильтрует filtered-массив |
| Процент побед/поражений | Frontend | Svelte: `wins / total * 100` |
| График 3: прогресс по баллам | Frontend | Chart.js — суммарные очки за тренировку по хронологии |
| Силуэт: нанесённый урон | Frontend | SVG с динамической заливкой зон по частоте попаданий |
| Силуэт: полученный урон | Frontend | SVG — берёт данные бойца B в бутах где A участвовал |
| Квик-блок: чаще всего использую приём | Frontend | Svelte: groupBy technique_a → max count |
| Квик-блок: чаще всего промахиваюсь | Frontend | Svelte: filter result_a=miss → groupBy technique → max |
| Квик-блок: чаще всего получаю от приёма | Frontend | Svelte: берёт technique_b из бутов с result_b=hit |
| Все блоки реагируют на фильтр таблицы | Frontend | Svelte: computed values из единого filtered-массива |
| Клик → → переход к видео | Frontend | Svelte Router |
| Дата первого зафиксированного боя | Frontend | Svelte: min(date) из массива боёв |

---

## 8. Управление пользователями

| Функция | Где | Инструмент |
|---|---|---|
| Создать пользователя (Admin) | Frontend → Backend | POST /api/admin/users |
| Хеширование пароля | **Backend** | Rust: bcrypt |
| Просмотр профиля | Frontend | GET /api/users/me |
| Редактирование имени | Frontend → Backend | PATCH /api/users/me |
| Смена пароля | Frontend → Backend | PATCH /api/users/me (bcrypt на бэкенде) |
| Загрузка/смена аватара | Frontend → Backend | POST /api/users/me/avatar (файл, хранится локально на сервере) |
| Проверка прав Admin | **Backend** | Rust: middleware читает is_admin из JWT |
| Удалить пользователя (Admin) | Frontend → Backend | DELETE /api/admin/users/:id |

---

## 9. Управление техниками (Admin)

| Функция | Где | Инструмент |
|---|---|---|
| Список всех техник | Frontend → Backend | GET /api/techniques |
| Добавить технику | Frontend → Backend | POST /api/admin/techniques |
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
    │◄── превью кадры (PNG) ──────────│    (один раз, кеш)      │
    │                                  │                         │
    │── GET /api/videos/:id ──────────►│── GET seafile_url ────►│
    │◄── метаданные + seafile_url ────│◄── временная ссылка ────│
    │                                  │                         │
    │── видео-поток ──────────────────────────────────────────►│
    │◄── байты видео ────────────────────────────────────────────│
    │   (браузер стримит напрямую)     │                         │
    │                                  │                         │
    │── WS: connect ─────────────────►│                         │
    │── WS: новый комментарий ────────►│── SQLite INSERT         │
    │◄── WS: broadcast всем зрителям ─│                         │
```

---

## WebSocket — только live-события

WebSocket используется **только** там, где нужно мгновенное обновление у нескольких пользователей:

| Событие | Направление |
|---|---|
| Новый комментарий | Backend → все кто смотрит это видео |
| Изменение/удаление схода | Backend → все кто смотрит это видео |
| Новое видео появилось в Seafile | Backend → все в галерее |

Всё остальное — REST.

---

## Что хранится где

| Данные | Где хранится |
|---|---|
| Видеофайлы | Seafile |
| Превью-кадры (PNG) | Локальная папка на сервере (`/data/previews/`) |
| Аватары пользователей | Локальная папка на сервере (`/data/avatars/`) |
| Пользователи, бойцы, права | SQLite |
| Видео-метаданные, seafile_path | SQLite |
| Буты (сходы) | SQLite |
| Техники | SQLite |
| Комментарии | SQLite |
| JWT-токены | Только у клиента (localStorage) |
