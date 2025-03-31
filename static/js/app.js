// Глобальные переменные
let tasks = [];
let tags = [];
let selectedTaskTags = [];
let taskToDeleteId = null;

// DOM-элементы
const taskTableBody = document.getElementById('tasksTableBody');
const taskForm = document.getElementById('taskForm');
const taskModal = new bootstrap.Modal(document.getElementById('taskModal'));
const deleteConfirmModal = new bootstrap.Modal(document.getElementById('deleteConfirmModal'));
const saveTaskBtn = document.getElementById('saveTaskBtn');
const confirmDeleteBtn = document.getElementById('confirmDeleteBtn');
const addTagBtn = document.getElementById('addTagBtn');
const tagsContainer = document.getElementById('tagsContainer');
const tagInput = document.getElementById('taskTags');
const availableTagsSpan = document.getElementById('availableTags');

// Фильтры
const filterStatus = document.getElementById('filterStatus');
const filterPriority = document.getElementById('filterPriority');
const filterCustomer = document.getElementById('filterCustomer');
const filterExecutor = document.getElementById('filterExecutor');
const filterTags = document.getElementById('filterTags');
const filterSearch = document.getElementById('filterSearch');
const applyFiltersBtn = document.getElementById('applyFilters');
const resetFiltersBtn = document.getElementById('resetFilters');

// Инициализация при загрузке страницы
document.addEventListener('DOMContentLoaded', () => {
    // Загрузка данных
    loadTasks();
    loadTags();
    
    // Обработчики событий для модальных окон
    setupEventListeners();
    
    // Запускаем таймер для обновления информации о просроченных днях
    setInterval(updateOverdueDays, 60000); // Обновляем каждую минуту
});

// Загрузка задач с сервера
function loadTasks() {
    fetch('/api/tasks')
        .then(response => {
            if (!response.ok) {
                throw new Error('Не удалось загрузить задачи');
            }
            return response.json();
        })
        .then(data => {
            tasks = data;
            renderTasksTable();
        })
        .catch(error => showNotification('Ошибка: ' + error.message, 'danger'));
}

// Загрузка тегов с сервера
function loadTags() {
    fetch('/api/tags')
        .then(response => {
            if (!response.ok) {
                throw new Error('Не удалось загрузить теги');
            }
            return response.json();
        })
        .then(data => {
            tags = data;
            updateAvailableTags();
        })
        .catch(error => showNotification('Ошибка: ' + error.message, 'danger'));
}

// Обновление отображения доступных тегов
function updateAvailableTags() {
    if (tags.length === 0) {
        availableTagsSpan.textContent = 'нет доступных тегов';
        return;
    }
    
    availableTagsSpan.textContent = tags.map(tag => tag.name).join(', ');
}

// Настройка обработчиков событий
function setupEventListeners() {
    // Обработчик для кнопки "Новое поручение"
    document.querySelector('[data-bs-target="#taskModal"]').addEventListener('click', () => {
        resetTaskForm();
        document.getElementById('taskModalLabel').textContent = 'Новое поручение';
        document.getElementById('taskStatus').disabled = true;
        document.getElementById('taskStatus').value = 'new';
    });
    
    // Обработчик для кнопки "Сохранить"
    saveTaskBtn.addEventListener('click', saveTask);
    
    // Обработчик для подтверждения удаления
    confirmDeleteBtn.addEventListener('click', deleteTask);
    
    // Обработчик для добавления тега
    addTagBtn.addEventListener('click', addTag);
    
    // Обработчик для поля ввода тега (при нажатии Enter)
    tagInput.addEventListener('keydown', event => {
        if (event.key === 'Enter') {
            event.preventDefault();
            addTag();
        }
    });
    
    // Обработчики для фильтров
    applyFiltersBtn.addEventListener('click', applyFilters);
    resetFiltersBtn.addEventListener('click', resetFilters);
}

// Отображение задач в таблице
function renderTasksTable() {
    taskTableBody.innerHTML = '';
    
    const filteredTasks = filterTasks();
    
    if (filteredTasks.length === 0) {
        const emptyRow = document.createElement('tr');
        emptyRow.innerHTML = '<td colspan="9" class="text-center py-3">Нет доступных поручений</td>';
        taskTableBody.appendChild(emptyRow);
        return;
    }
    
    filteredTasks.forEach(task => {
        const row = document.createElement('tr');
        
        // Вычисляем количество просроченных дней
        let overdueDays = '';
        let overdueClass = '';
        
        if (task.status !== 'completed' && task.status !== 'cancelled' && task.due_date) {
            const dueDate = new Date(task.due_date);
            const now = new Date();
            
            if (now > dueDate) {
                const diffTime = Math.abs(now - dueDate);
                const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
                overdueDays = `${diffDays} ${getDayWordForm(diffDays)}`;
                overdueClass = 'overdue';
            }
        }
        
        // Находим имена заказчика и исполнителя
        const customerName = getCustomerName(task.customer_id);
        const executorName = getExecutorName(task.executor_id);
        
        // Формируем HTML для тегов
        const tagsHtml = task.tags.map(tag => 
            `<span class="tag-badge">${tag.name}</span>`
        ).join('');
        
        // Формируем статус и приоритет с соответствующими стилями
        const statusText = getStatusText(task.status);
        const priorityText = getPriorityText(task.priority);
        const statusClass = `status-${task.status}`;
        const priorityClass = `priority-${task.priority}`;
        
        // Формируем дату выполнения
        const dueDateText = task.due_date 
            ? new Date(task.due_date).toLocaleString('ru-RU')
            : 'Не указан';
        
        row.innerHTML = `
            <td>${task.title}</td>
            <td><span class="badge ${statusClass}">${statusText}</span></td>
            <td><span class="badge ${priorityClass}">${priorityText}</span></td>
            <td>${customerName}</td>
            <td>${executorName}</td>
            <td>${dueDateText}</td>
            <td class="${overdueClass}">${overdueDays}</td>
            <td>${tagsHtml}</td>
            <td>
                <button class="btn btn-sm btn-outline-primary action-btn edit-task-btn" data-id="${task.id}">
                    <i class="bi bi-pencil"></i>
                </button>
                <button class="btn btn-sm btn-outline-danger action-btn delete-task-btn" data-id="${task.id}">
                    <i class="bi bi-trash"></i>
                </button>
            </td>
        `;
        
        taskTableBody.appendChild(row);
    });
    
    // Добавляем обработчики для кнопок редактирования и удаления
    document.querySelectorAll('.edit-task-btn').forEach(btn => {
        btn.addEventListener('click', () => editTask(btn.dataset.id));
    });
    
    document.querySelectorAll('.delete-task-btn').forEach(btn => {
        btn.addEventListener('click', () => showDeleteConfirm(btn.dataset.id));
    });
}

// Функция для фильтрации задач
function filterTasks() {
    return tasks.filter(task => {
        // Фильтр по статусу
        if (filterStatus.value && task.status !== filterStatus.value) {
            return false;
        }
        
        // Фильтр по приоритету
        if (filterPriority.value && task.priority !== filterPriority.value) {
            return false;
        }
        
        // Фильтр по заказчику
        if (filterCustomer.value && task.customer_id !== filterCustomer.value) {
            return false;
        }
        
        // Фильтр по исполнителю
        if (filterExecutor.value && task.executor_id !== filterExecutor.value) {
            return false;
        }
        
        // Фильтр по тегам
        if (filterTags.value.trim()) {
            const filterTagsList = filterTags.value.split(',').map(tag => tag.trim().toLowerCase());
            const taskTagNames = task.tags.map(tag => tag.name.toLowerCase());
            
            if (!filterTagsList.some(tag => taskTagNames.includes(tag))) {
                return false;
            }
        }
        
        // Фильтр по поиску
        if (filterSearch.value.trim()) {
            const searchTerm = filterSearch.value.trim().toLowerCase();
            return (
                task.title.toLowerCase().includes(searchTerm) ||
                task.description.toLowerCase().includes(searchTerm)
            );
        }
        
        return true;
    });
}

// Применение фильтров
function applyFilters() {
    renderTasksTable();
}

// Сброс фильтров
function resetFilters() {
    filterStatus.value = '';
    filterPriority.value = '';
    filterCustomer.value = '';
    filterExecutor.value = '';
    filterTags.value = '';
    filterSearch.value = '';
    renderTasksTable();
}

// Обновление количества просроченных дней
function updateOverdueDays() {
    renderTasksTable();
}

// Редактирование задачи
function editTask(taskId) {
    const task = tasks.find(t => t.id === taskId);
    if (!task) return;
    
    // Заполняем форму данными задачи
    document.getElementById('taskId').value = task.id;
    document.getElementById('taskTitle').value = task.title;
    document.getElementById('taskDescription').value = task.description;
    document.getElementById('taskCustomer').value = task.customer_id;
    document.getElementById('taskExecutor').value = task.executor_id;
    document.getElementById('taskPriority').value = task.priority;
    document.getElementById('taskStatus').value = task.status;
    document.getElementById('taskStatus').disabled = false;
    
    // Устанавливаем срок выполнения
    if (task.due_date) {
        const dueDate = new Date(task.due_date);
        const offset = dueDate.getTimezoneOffset() * 60000;
        const localDueDate = new Date(dueDate.getTime() - offset);
        document.getElementById('taskDueDate').value = localDueDate.toISOString().slice(0, 16);
    } else {
        document.getElementById('taskDueDate').value = '';
    }
    
    // Заполняем теги
    selectedTaskTags = [...task.tags];
    renderSelectedTags();
    
    // Обновляем заголовок модального окна
    document.getElementById('taskModalLabel').textContent = 'Редактирование поручения';
    
    // Открываем модальное окно
    taskModal.show();
}

// Добавление тега в выбранные
function addTag() {
    const tagName = tagInput.value.trim();
    if (!tagName) return;
    
    // Проверяем, не добавлен ли уже такой тег
    if (selectedTaskTags.some(tag => tag.name.toLowerCase() === tagName.toLowerCase())) {
        showNotification('Этот тег уже добавлен', 'warning');
        tagInput.value = '';
        return;
    }
    
    // Проверяем, существует ли такой тег в системе
    let tag = tags.find(t => t.name.toLowerCase() === tagName.toLowerCase());
    
    // Если тег не существует, создаем новый
    if (!tag) {
        tag = { id: 'temp_' + Date.now(), name: tagName };
    }
    
    selectedTaskTags.push(tag);
    renderSelectedTags();
    tagInput.value = '';
}

// Удаление тега из выбранных
function removeTag(tagId) {
    selectedTaskTags = selectedTaskTags.filter(tag => tag.id !== tagId);
    renderSelectedTags();
}

// Отображение выбранных тегов
function renderSelectedTags() {
    tagsContainer.innerHTML = '';
    
    selectedTaskTags.forEach(tag => {
        const tagElement = document.createElement('span');
        tagElement.className = 'tag-badge';
        tagElement.textContent = tag.name;
        
        const removeBtn = document.createElement('span');
        removeBtn.className = 'tag-remove';
        removeBtn.textContent = '×';
        removeBtn.addEventListener('click', () => removeTag(tag.id));
        
        tagElement.appendChild(removeBtn);
        tagsContainer.appendChild(tagElement);
    });
}

// Сохранение задачи
function saveTask() {
    // Проверяем валидность формы
    if (!taskForm.checkValidity()) {
        taskForm.reportValidity();
        return;
    }
    
    // Собираем данные из формы
    const taskId = document.getElementById('taskId').value;
    const title = document.getElementById('taskTitle').value;
    const description = document.getElementById('taskDescription').value;
    const customer_id = document.getElementById('taskCustomer').value;
    const executor_id = document.getElementById('taskExecutor').value;
    const priority = document.getElementById('taskPriority').value;
    const status = document.getElementById('taskStatus').value;
    const dueDateInput = document.getElementById('taskDueDate').value;
    
    // Преобразуем теги в формат для API
    const tagNames = selectedTaskTags.map(tag => tag.name);
    
    // Создаем объект с данными задачи
    const taskData = {
        title,
        description,
        priority,
        customer_id,
        executor_id,
        tags: tagNames
    };
    
    // Добавляем статус для редактирования
    if (taskId) {
        taskData.status = status;
    }
    
    // Добавляем срок выполнения, если указан
    if (dueDateInput) {
        taskData.due_date = new Date(dueDateInput).toISOString();
    }
    
    // Определяем метод и URL в зависимости от того, создаем или редактируем задачу
    const method = taskId ? 'PUT' : 'POST';
    const url = taskId ? `/api/tasks/${taskId}` : '/api/tasks';
    
    // Отправляем запрос на сервер
    fetch(url, {
        method,
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(taskData),
    })
    .then(response => {
        if (!response.ok) {
            throw new Error('Не удалось сохранить задачу');
        }
        return response.json();
    })
    .then(() => {
        // Закрываем модальное окно
        taskModal.hide();
        
        // Перезагружаем список задач и тегов
        loadTasks();
        loadTags();
        
        // Показываем уведомление об успешном сохранении
        const message = taskId ? 'Поручение успешно обновлено' : 'Новое поручение создано';
        showNotification(message, 'success');
    })
    .catch(error => showNotification('Ошибка: ' + error.message, 'danger'));
}

// Показать подтверждение удаления
function showDeleteConfirm(taskId) {
    taskToDeleteId = taskId;
    deleteConfirmModal.show();
}

// Удаление задачи
function deleteTask() {
    if (!taskToDeleteId) return;
    
    fetch(`/api/tasks/${taskToDeleteId}`, {
        method: 'DELETE',
    })
    .then(response => {
        if (!response.ok) {
            throw new Error('Не удалось удалить задачу');
        }
        
        // Закрываем модальное окно
        deleteConfirmModal.hide();
        
        // Перезагружаем список задач
        loadTasks();
        
        // Показываем уведомление об успешном удалении
        showNotification('Поручение успешно удалено', 'success');
    })
    .catch(error => showNotification('Ошибка: ' + error.message, 'danger'));
}

// Сброс формы
function resetTaskForm() {
    document.getElementById('taskId').value = '';
    taskForm.reset();
    selectedTaskTags = [];
    renderSelectedTags();
}

// Показ уведомления
function showNotification(message, type = 'info') {
    // Создаем элемент уведомления
    const notification = document.createElement('div');
    notification.className = `alert alert-${type} alert-dismissible fade show alert-appear`;
    notification.innerHTML = `
        ${message}
        <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
    `;
    
    // Добавляем уведомление на страницу
    const container = document.querySelector('main .container');
    container.insertBefore(notification, container.firstChild);
    
    // Автоматически скрываем через 5 секунд
    setTimeout(() => {
        notification.classList.remove('show');
        setTimeout(() => notification.remove(), 300);
    }, 5000);
}

// Вспомогательные функции
function getStatusText(status) {
    const statuses = {
        'new': 'Новая',
        'in_progress': 'В работе',
        'completed': 'Завершена',
        'cancelled': 'Отменена'
    };
    return statuses[status] || status;
}

function getPriorityText(priority) {
    const priorities = {
        'low': 'Низкий',
        'medium': 'Средний',
        'high': 'Высокий',
        'critical': 'Критический'
    };
    return priorities[priority] || priority;
}

function getCustomerName(customerId) {
    // В реальном приложении это будет запрос к API или использование кэшированных данных пользователей
    return `Заказчик ${customerId.substring(0, 6)}`;
}

function getExecutorName(executorId) {
    // В реальном приложении это будет запрос к API или использование кэшированных данных пользователей
    return `Исполнитель ${executorId.substring(0, 6)}`;
}

function getDayWordForm(days) {
    if (days % 100 >= 11 && days % 100 <= 14) {
        return 'дней';
    }
    
    switch (days % 10) {
        case 1:
            return 'день';
        case 2:
        case 3:
        case 4:
            return 'дня';
        default:
            return 'дней';
    }
}
