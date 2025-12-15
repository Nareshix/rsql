CREATE TABLE products (
    product_id INT,
    product_name TEXT,
    description TEXT,
    price REAL,
    is_in_stock BOOL NOT NULL
);
CREATE TABLE employees (
    employee_id INT,
    full_name TEXT,
    department TEXT,
    age INT,
    hourly_wage REAL,
    is_active BOOL
);
CREATE TABLE weather_log (
    log_id INT,
    station_location TEXT,
    temperature_celsius REAL,
    wind_speed_ms REAL,
    is_raining BOOL
);
CREATE TABLE Persons (
    PersonID INTEGER,
    LastName TEXT,
    FirstName TEXT,
    Address TEXT,
    City TEXT
);