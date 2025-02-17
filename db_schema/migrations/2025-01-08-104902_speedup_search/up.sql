-- Your SQL goes here

-- Миграция для оптимизации поиска

-- Добавляем индекс для ускорения фильтрации по id
CREATE INDEX IF NOT EXISTS idx_servers_id
    ON public.servers USING btree (id);

-- Добавляем индекс для сортировки по id (DESC)
CREATE INDEX IF NOT EXISTS idx_servers_id_desc
    ON public.servers USING btree (id DESC);
