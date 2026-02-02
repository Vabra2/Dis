# 🔐 Dis - Анонимная Чат-Платформа

> Self-hosted, зашифрованная, анонимная платформа для общения на базе [Revolt](https://github.com/revoltchat)

![License](https://img.shields.io/badge/license-AGPL--3.0-blue)
![Docker](https://img.shields.io/badge/docker-ready-brightgreen)
![E2E](https://img.shields.io/badge/encryption-E2E-green)

## ✨ Особенности

- 🔒 **E2E Шифрование** — Signal Protocol для всех сообщений
- 👤 **Полная анонимность** — никаких обязательных персональных данных
- 🧅 **Tor поддержка** — доступ через .onion
- 🎙️ **Голосовые каналы** — WebRTC с SRTP шифрованием
- 📧 **Опциональный email** — только для восстановления, хранится зашифрованным
- 🐳 **Docker деплой** — один `docker-compose up` для запуска
- 📝 **Минимальные логи** — автоматическая анонимизация и удаление

## 🚀 Быстрый старт

```bash
# Клонируем репозиторий
git clone https://github.com/Vabra2/Dis.git
cd Dis

# Запускаем установку
chmod +x scripts/setup.sh
./scripts/setup.sh

# Редактируем конфигурацию
nano .env

# Запускаем
docker-compose up -d
```

## 📁 Структура проекта

```
Dis/
├── docker-compose.yml      # Оркестрация сервисов
├── Caddyfile              # Reverse proxy + SSL
├── encryption/            # E2E шифрование (Rust)
├── auth/                  # Анонимная авторизация (Rust)
├── email/                 # Email сервис (Python)
├── logs/                  # Анонимизация логов
├── scripts/               # Скрипты установки
├── tor/                   # Tor Hidden Service
└── docs/                  # Документация
```

## 🔧 Требования

- Docker 20.10+
- Docker Compose 2.0+
- 2GB RAM минимум
- Домен (опционально, для HTTPS)

## 📖 Документация

- [Установка](docs/INSTALL.md)
- [Безопасность](docs/SECURITY.md)
- [Анонимность](docs/ANONYMITY.md)
- [Архитектура](docs/ARCHITECTURE.md)

## 🛡️ Безопасность

Этот проект использует:
- **Signal Protocol** для E2E шифрования
- **Argon2** для хэширования паролей
- **AES-256-GCM** для шифрования данных at-rest
- **TLS 1.3** для транспортного шифрования

## ⚖️ Лицензия

AGPL-3.0 — см. [LICENSE](LICENSE)

---

**⚠️ Disclaimer:** Этот проект создан для легального использования. Операторы платформы несут ответственность за соблюдение законодательства.