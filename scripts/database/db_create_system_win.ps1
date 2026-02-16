# ============================================
# ShareUSTC Database System Initialization Script (Windows)
# Requires Administrator privileges
# Function: Create database and user
# ============================================

#Requires -RunAsAdministrator

# Configuration
$DB_NAME = "shareustc"
$DB_USER = "shareustc_app" 
$DB_PASSWORD = "114514" # Note: In production, use environment variables or secure vaults for credentials
$POSTGRES_USER = "postgres"

# Color output function
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

Write-ColorOutput Green "=== ShareUSTC Database System Initialization (Windows) ==="
Write-Output ""

# Find psql
function Find-Psql {
    $psqlCmd = Get-Command psql -ErrorAction SilentlyContinue
    if ($psqlCmd) {
        return $psqlCmd.Source
    }
    
    $commonPaths = @(
        "C:\Program Files\PostgreSQL\*\bin\psql.exe",
        "C:\Program Files (x86)\PostgreSQL\*\bin\psql.exe"
    )
    
    foreach ($path in $commonPaths) {
        $matches = Get-ChildItem -Path $path -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($matches) {
            $env:Path += ";" + $matches.DirectoryName
            return $matches.FullName
        }
    }
    
    return $null
}

$psqlPath = Find-Psql
if (-not $psqlPath) {
    Write-ColorOutput Red "Error: psql command not found. Please install PostgreSQL and add to PATH."
    exit 1
}

Write-ColorOutput Yellow "Using psql: $psqlPath"
Write-Output ""

# Check PostgreSQL service
Write-ColorOutput Yellow "Step 1/4: Checking PostgreSQL service status..."
$service = Get-Service -Name "postgresql*" -ErrorAction SilentlyContinue | Select-Object -First 1

if ($service -and $service.Status -eq "Running") {
    Write-ColorOutput Green "  PostgreSQL service is running"
} else {
    Write-ColorOutput Yellow "  Starting PostgreSQL service..."
    if ($service) {
        Start-Service $service.Name
        Set-Service $service.Name -StartupType Automatic
        Write-ColorOutput Green "  PostgreSQL service started"
    } else {
        Write-ColorOutput Red "  Error: PostgreSQL service not found. Please ensure PostgreSQL is installed."
        exit 1
    }
}

Write-Output ""

# Prompt for postgres password
Write-ColorOutput Yellow "Please enter PostgreSQL '$POSTGRES_USER' user password (default is often empty or 'postgres'):"
$postgresPassword = Read-Host -AsSecureString "PostgreSQL postgres password"
$BSTR = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($postgresPassword)
$plainPostgresPassword = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($BSTR)

$env:PGPASSWORD = $plainPostgresPassword

# Test postgres connection
Write-ColorOutput Yellow "Testing postgres connection..."
try {
    $testResult = & $psqlPath -U $POSTGRES_USER -d postgres -c "SELECT 1;" 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Connection failed"
    }
    Write-ColorOutput Green "  Connection successful"
} catch {
    Write-ColorOutput Red "  Error: Cannot connect to PostgreSQL as '$POSTGRES_USER'."
    Write-ColorOutput Red "  Please check the password and ensure PostgreSQL is running."
    exit 1
}

Write-Output ""

# Create database user
Write-ColorOutput Yellow "Step 2/4: Creating database user '$DB_USER'..."
try {
    $userExists = & $psqlPath -U $POSTGRES_USER -d postgres -t -c "SELECT 1 FROM pg_roles WHERE rolname='$DB_USER';" 2>&1 | Out-String
    if ($userExists.Trim() -eq "1") {
        Write-ColorOutput Yellow "  User '$DB_USER' already exists, skipping creation"
    } else {
        & $psqlPath -U $POSTGRES_USER -d postgres -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';" 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Write-ColorOutput Green "  User '$DB_USER' created successfully"
        } else {
            throw "Failed to create user"
        }
    }
} catch {
    Write-ColorOutput Red "  Error: Failed to create user"
    Write-ColorOutput Red "  $_"
    exit 1
}

Write-Output ""

# Create database
Write-ColorOutput Yellow "Step 3/4: Creating database '$DB_NAME'..."
try {
    $dbExists = & $psqlPath -U $POSTGRES_USER -d postgres -t -c "SELECT 1 FROM pg_database WHERE datname='$DB_NAME';" 2>&1 | Out-String
    if ($dbExists.Trim() -eq "1") {
        Write-ColorOutput Yellow "  Database '$DB_NAME' already exists, skipping creation"
    } else {
        & $psqlPath -U $POSTGRES_USER -d postgres -c "CREATE DATABASE $DB_NAME OWNER $DB_USER ENCODING 'UTF8' LC_COLLATE 'C' LC_CTYPE 'C' TEMPLATE template0;" 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Write-ColorOutput Green "  Database '$DB_NAME' created successfully"
        } else {
            throw "Failed to create database"
        }
    }
} catch {
    Write-ColorOutput Red "  Error: Failed to create database"
    Write-ColorOutput Red "  $_"
    exit 1
}

Write-Output ""

# Grant permissions
Write-ColorOutput Yellow "Step 4/4: Granting permissions..."
try {
    # Grant connect permission
    & $psqlPath -U $POSTGRES_USER -d postgres -c "GRANT CONNECT ON DATABASE $DB_NAME TO $DB_USER;" 2>&1 | Out-Null
    
    # Grant schema permissions
    & $psqlPath -U $POSTGRES_USER -d $DB_NAME -c "GRANT USAGE ON SCHEMA public TO $DB_USER;" 2>&1 | Out-Null
    & $psqlPath -U $POSTGRES_USER -d $DB_NAME -c "GRANT CREATE ON SCHEMA public TO $DB_USER;" 2>&1 | Out-Null
    
    # Enable pgcrypto extension
    & $psqlPath -U $POSTGRES_USER -d $DB_NAME -c "CREATE EXTENSION IF NOT EXISTS pgcrypto;" 2>&1 | Out-Null
    
    Write-ColorOutput Green "  Permissions granted successfully"
} catch {
    Write-ColorOutput Red "  Error: Failed to grant permissions"
    Write-ColorOutput Red "  $_"
    exit 1
}

Write-Output ""
Write-ColorOutput Green "=== System Initialization Completed ==="
Write-Output ""
Write-Output "Database Information:"
Write-Output "  Database Name: $DB_NAME"
Write-Output "  Username:      $DB_USER"
Write-Output "  Password:      $DB_PASSWORD"
Write-Output ""
Write-ColorOutput Yellow "Next Step: Run database table initialization"
Write-Output "  .\db_init_tables_win.ps1"
Write-Output ""
Write-Output "Or use Python script:"
Write-Output "  python init_db.py"
