# Minesweeper API

HTTP API для управления игрой.

## Запуск

```bash
cargo run
```

Сервер стартует на http://127.0.0.1:8080

## API Endpoints

| Method | URL | Описание |
|--------|-----|----------|
| GET | `/json` | Получить состояние игры в JSON |
| POST | `/click?x=0&y=0` | Левый клик по клетке (x - колонка, y - строка) |
| POST | `/flag?x=0&y=0` | Поставить/убрать флаг |
| POST | `/restart` | Начать новую игру |

## Примеры

```bash
# Получить состояние
curl http://127.0.0.1:8080/json

# Кликнуть на клетку (5, 3)
curl -X POST "http://127.0.0.1:8080/click?x=5&y=3"

# Поставить флаг на клетку (5, 3)
curl -X POST "http://127.0.0.1:8080/flag?x=5&y=3"

# Начать заново
curl -X POST http://127.0.0.1:8080/restart
```

## Формат ответа /json

```json
{
  "width": 30,
  "height": 16,
  "cells": [
    [
      {"cell": "empty", "is_opened": false, "is_labeled": false},
      {"cell": "mine", "is_opened": false, "is_labeled": true},
      {"cell": "1", "is_opened": true, "is_labeled": false}
    }
  ],
  "time": 42,
  "status": "",
  "mines_found": 5
}
```

- `cell`: `"mine"`, `"empty"`, или цифра (0-8)
- `is_opened`: открыта ли клетка
- `is_labeled`: установлен ли флаг
