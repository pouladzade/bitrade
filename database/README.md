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

### **Install `libpq` Development Package**

Depending on the operating system, install the required package:

#### **Ubuntu/Debian**

```sh
sudo apt update
sudo apt install libpq-dev
```

#### **Fedora**

```sh
sudo dnf install postgresql-devel
```

#### **Arch Linux**

```sh
sudo pacman -S postgresql-libs
```

#### **macOS (Homebrew)**

```sh
brew install postgresql
```

#### **Windows (Using vcpkg or MSYS2)**

If using `vcpkg`:

```sh
vcpkg install libpq
```

Or if using MSYS2:

```sh
pacman -S mingw-w64-x86_64-postgresql
```
