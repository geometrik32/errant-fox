# Errant Fox

<p align="center">
  <img src="docs/assets/logo.png" width="200" alt="Errant Fox Logo">
</p>

<p align="center">
  <strong>Современная платформа для анализа видео спаррингов HEMA (Historical European Martial Arts)</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Backend-Rust-black?style=for-the-badge&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Frontend-Svelte_5-orange?style=for-the-badge&logo=svelte" alt="Svelte 5">
  <img src="https://img.shields.io/badge/Database-SQLite-blue?style=for-the-badge&logo=sqlite" alt="SQLite">
  <img src="https://img.shields.io/badge/Platform-TrueNAS-0058A3?style=for-the-badge&logo=truenas" alt="TrueNAS">
  <img src="https://img.shields.io/badge/Deployment-Docker-2496ED?style=for-the-badge&logo=docker" alt="Docker">
</p>

---

## 🦊 О проекте

**Errant Fox** — это специализированный инструмент для фехтовальных клубов, предназначенный для детального разбора видеозаписей спаррингов (на данный момент фокусируется на длинном мече). Система позволяет бойцам просматривать записи, размечать обмены, тегировать техники и отслеживать личную статистику.

Проект разработан для развертывания на домашнем сервере (TrueNAS) с интеграцией облачного хранилища Seafile для видеофайлов.

## 🚀 Быстрый старт

### Требования
- Docker & Docker Compose
- Seafile Server (для хранения видео)

### Запуск
1. Склонируйте репозиторий.
2. Настройте переменные окружения в файле `.env`.
3. Запустите проект из папки `infra`:
   ```bash
   cd infra
   docker-compose up -d
   ```

## 📂 Структура проекта

- `backend/` — Серверная часть приложения на Rust (Axum, Diesel).
- `frontend/` — Клиентская часть на Svelte 5 + Vite.
- `docs/` — Техническая документация, API Reference и гайды.
- `scripts/` — Скрипты автоматизации и вспомогательные инструменты.
- `infra/` — Конфигурации для развертывания (Docker, Nginx).

## 📚 Документация

Вся детальная информация находится в директории `docs/`:

- 📋 [Требования (Requirements)](./docs/requirements.md) — Полная техническая спецификация.
- 📐 [Архитектура (Architecture)](./docs/architecture.md) — Описание компонентов и взаимодействия.
- 💾 [База данных (Database)](./docs/database.md) — Схема таблиц SQLite.
- 🔌 [API Reference](./docs/api.md) — Документация REST-эндпоинтов.
- 🚢 [Deployment Guide](./deploy.md) — Инструкции по деплою.

---

<p align="center">
  <i>Разработано с акцентом на производительность и удобство анализа.</i>
</p>
