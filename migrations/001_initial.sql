-- Создание таблицы пользователей
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    role TEXT NOT NULL
);

-- Создание таблицы тегов
CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

-- Создание таблицы задач
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL,
    priority TEXT NOT NULL,
    customer_id TEXT NOT NULL,
    executor_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    due_date TEXT,
    completed_at TEXT,
    FOREIGN KEY (customer_id) REFERENCES users (id),
    FOREIGN KEY (executor_id) REFERENCES users (id)
);

-- Создание таблицы связей между задачами и тегами
CREATE TABLE IF NOT EXISTS task_tags (
    task_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (task_id, tag_id),
    FOREIGN KEY (task_id) REFERENCES tasks (id),
    FOREIGN KEY (tag_id) REFERENCES tags (id)
);

-- Вставка тестовых данных (заказчики)
INSERT INTO users (id, name, email, role) VALUES
('cust_1', 'Иванов Иван', 'ivanov@example.com', 'customer'),
('cust_2', 'Петров Петр', 'petrov@example.com', 'customer'),
('cust_3', 'Сидорова Елена', 'sidorova@example.com', 'customer');

-- Вставка тестовых данных (исполнители)
INSERT INTO users (id, name, email, role) VALUES
('exec_1', 'Смирнов Алексей', 'smirnov@example.com', 'executor'),
('exec_2', 'Козлова Мария', 'kozlova@example.com', 'executor'),
('exec_3', 'Новиков Дмитрий', 'novikov@example.com', 'executor');

-- Вставка тестовых данных (теги)
INSERT INTO tags (id, name) VALUES
('tag_1', 'Срочно'),
('tag_2', 'Отчет'),
('tag_3', 'Встреча'),
('tag_4', 'Документация'),
('tag_5', 'Разработка');

-- Вставка тестовых данных (задачи)
INSERT INTO tasks (id, title, description, status, priority, customer_id, executor_id, created_at, due_date, completed_at) VALUES
('task_1', 'Подготовить отчет за квартал', 'Необходимо подготовить финансовый отчет за 1 квартал 2025 года', 'new', 'high', 'cust_1', 'exec_1', '2025-03-20T10:00:00Z', '2025-04-05T18:00:00Z', NULL),
('task_2', 'Организовать встречу с клиентом', 'Организовать встречу с представителями компании "Альфа"', 'in_progress', 'medium', 'cust_2', 'exec_2', '2025-03-25T14:30:00Z', '2025-04-10T15:00:00Z', NULL),
('task_3', 'Обновить документацию проекта', 'Внести изменения в техническую документацию проекта "Бета"', 'completed', 'low', 'cust_3', 'exec_3', '2025-03-15T09:15:00Z', '2025-03-30T18:00:00Z', '2025-03-28T16:45:00Z'),
('task_4', 'Разработать новый функционал', 'Реализовать модуль статистики для системы управления', 'in_progress', 'critical', 'cust_1', 'exec_3', '2025-03-22T11:20:00Z', '2025-04-15T18:00:00Z', NULL);

-- Вставка тестовых данных (связи задач и тегов)
INSERT INTO task_tags (task_id, tag_id) VALUES
('task_1', 'tag_1'),
('task_1', 'tag_2'),
('task_2', 'tag_3'),
('task_3', 'tag_4'),
('task_4', 'tag_5'),
('task_4', 'tag_1');
