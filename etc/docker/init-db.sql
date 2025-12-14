CREATE DATABASE IF NOT EXISTS order_db;
CREATE DATABASE IF NOT EXISTS inventory_db;
CREATE DATABASE IF NOT EXISTS payment_db;

GRANT ALL PRIVILEGES ON order_db.* TO 'ecuser'@'%';
GRANT ALL PRIVILEGES ON inventory_db.* TO 'ecuser'@'%';
GRANT ALL PRIVILEGES ON payment_db.* TO 'ecuser'@'%';
