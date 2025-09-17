# Enterprise Document and Case Management Platform

An enterprise-grade document and case management platform built with Rust, featuring:

- **Document Management**: Upload, version, categorize and search documents
- **Case Management**: Create, track, and manage cases with full audit trails
- **User Management**: Authentication, authorization with role-based access
- **RESTful API**: Complete REST API for integration with other systems
- **Database Integration**: SQLite for development, PostgreSQL ready for production

## Features

### Document Management
- Document upload and storage
- Version control and history
- Metadata and tagging system
- Full-text search capabilities
- Status tracking (Draft, Review, Approved, Published, Archived)
- Access control and permissions

### Case Management
- Case creation and lifecycle management
- Status tracking (Open, In Progress, Under Review, Resolved, Closed)
- Priority levels (Low, Medium, High, Critical)
- Assignment and delegation
- Document attachment to cases
- Complete audit trail and history

### Enterprise Features
- Role-based access control (Admin, Manager, User, ReadOnly)
- JWT-based authentication
- RESTful API with comprehensive endpoints
- Database migrations
- Health check endpoints
- Structured logging
- Configuration management

## Quick Start

### Prerequisites
- Rust 1.70+ installed
- SQLite for development (PostgreSQL for production)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/harborgrid-justin/rust.git
cd rust
```

2. Copy and configure environment:
```bash
cp .env.example .env
# Edit .env with your configuration
```

3. Build and run:
```bash
cargo run
```

The server will start on `http://localhost:3000`

## API Endpoints

### Authentication
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login user

### Health Checks
- `GET /health` - Health check
- `GET /ready` - Readiness check

### Document Management
- `GET /api/documents` - List documents
- `POST /api/documents` - Create document
- `GET /api/documents/{id}` - Get document
- `POST /api/documents/{id}` - Update document
- `POST /api/documents/{id}/upload` - Upload file
- `GET /api/documents/{id}/download` - Download file

### Case Management
- `GET /api/cases` - List cases
- `POST /api/cases` - Create case
- `GET /api/cases/{id}` - Get case
- `POST /api/cases/{id}` - Update case
- `GET /api/cases/{id}/documents` - Get case documents
- `POST /api/cases/{id}/documents/{doc_id}` - Add document to case

## Configuration

Configuration is managed through environment variables:

- `DATABASE_URL` - Database connection string
- `SERVER_ADDRESS` - Server bind address
- `JWT_SECRET` - Secret key for JWT tokens
- `UPLOAD_DIR` - Directory for file uploads
- `MAX_FILE_SIZE` - Maximum file upload size

## Development

### Running Tests
```bash
cargo test
```

### Database Migrations
Migrations are automatically run on startup. Migration files are in the `migrations/` directory.

### Building for Production
```bash
cargo build --release
```

## Architecture

The application follows a layered architecture:

- **Handlers**: HTTP request handlers
- **Services**: Business logic layer
- **Models**: Data models and DTOs
- **Utils**: Utility functions and helpers
- **Config**: Configuration management

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
