### How to Use This Code:

1. **Install Diesel and Setup Migration Files**:

    ```bash
    # Linux/MacOS
    curl --proto '=https' --tlsv1.2 -LsSf https://github.com/diesel-rs/diesel/releases/latest/download/diesel_cli-installer.sh | sh

    # Windows (powershell)
    Set-ExecutionPolicy RemoteSigned -scope CurrentUser
    irm https://github.com/diesel-rs/diesel/releases/latest/download/diesel_cli-installer.ps1 | iex
    ```

   ```bash
   diesel setup --database-url=postgres://username:password@localhost/database_name
   diesel migration generate create_initial_tables
   ```

2. **Add SQL to Migration Files**:
   Copy the SQL schema into the generated up.sql file, and add DROP TABLE statements to down.sql.

3. **Run Migrations**:

   ```bash
   diesel migration run
   ```

4. **In Your Application**:

   ```rust
   use spot_database::*;

   fn main()
   ```
