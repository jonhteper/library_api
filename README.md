# Library API

Una API de gestión de biblioteca desarrollada en Rust, que ofrece tanto una interfaz RESTful como una CLI para administrar libros y gestionar accesos mediante claves API.

## Características

- API RESTful para la gestión de bibliotecas
- Herramienta CLI para administración de claves API
- Base de datos SQLite para persistencia
- Sistema de gestión de claves API para autenticación
- Validación de datos robusta
- Configuración flexible mediante variables de entorno
- Documentación OpenAPI

## Requisitos previos

- [Rust](https://www.rust-lang.org/tools/install) (edición 2024)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (incluido con Rust)
- [Make](https://www.gnu.org/software/make/) (opcional, para configuración automática)

## Instalación

Clona el repositorio:

```bash
git clone https://github.com/jonhteper/library_api

cd library_api
```

## Configuración

### Automática
Si utilizas Make para compilar el proyecto utiliza el siguiente comando para configurar automáticamente
```bash
make prepare
```
o
```bash
make
```
para compilar en modo release y configurar automáticamente.

### Manual
El proyecto utiliza un archivo `.env` para la configuración. Debe tener el siguiete esquema:

```
DATABASE_URL=sqlite:library_api.db
API_PORT=8080
RUST_LOG=info
```
Puedes modificar estos valores según tus necesidades.

El proyecto utiliza un archivo `.db` para la persistencia de datos, es indispensable que el archivo exista y tenga permisos adecuados antes de ejecutar los binarios.



## Compilación

### Usando Make

Al utilizar Make como método de compilación se utilizará el directorio `packages` para desplegar la API y la CLI, además se crearán los archivos `library_api.db` y `.env`.

Para compilar la API y la CLI en modo release:

```bash
make
```

Para compilar solo la API en modo desarrollo:

```bash
make dev
```

Para compilar solo la CLI en modo desarrollo:

```bash
make dev-cli
```

### Usando Cargo directamente

Compilar la API:

```bash
cargo build --package library_api --bin library_api
```

Compilar la CLI:

```bash
cargo build --package library_api --bin library_cli --features cli
```

## Ejecución

### API RESTful

Después de compilar, puedes ejecutar la API:

```bash
# Si compilaste con make
./packages/library-api

# Si compilaste directamente con cargo
./target/debug/library_api
```

La API estará disponible en `http://localhost:8080` (o el puerto configurado en `.env`).

### CLI

Ejecuta la herramienta CLI para administrar las claves API:

```bash
# Si compilaste con make
./packages/library-cli [COMANDOS]

# Si compilaste directamente con cargo
./target/debug/library_cli [COMANDOS]
```

Comandos disponibles en la CLI:

- `gen` (o `--gen`): Genera una nueva clave API y la muestra en pantalla
  ```bash
  ./packages/library-cli gen
  ```

- `delete` (o `--del`): Elimina una clave API existente por su ID
  ```bash
  ./packages/library-cli delete <APIKEY-ID>
  ```

## Pruebas

Ejecuta las pruebas unitarias con:

```bash
make test
# o
cargo test -- --show-output
```

Ejecuta las pruebas unitarias y de integración con:

```bash
make all-tests
# o
cargo test --features integration-tests -- --show-output
```

## Análisis de código

Para ejecutar Clippy y verificar el código:

```bash
make lint
# o
cargo clippy
```

## Estructura del proyecto

- `bin/`: Contiene los puntos de entrada para la API y la CLI
- `src/`: Código fuente principal
  - `api_keys/`: Gestión de claves de API
  - `books/`: Gestión de libros y operaciones CRUD
  - `config.rs`: Configuración de la aplicación
  - `errors.rs`: Manejo de errores
  - `init.rs`: Inicialización de servicios
  - `server.rs`: Configuración del servidor web
- `tests/`: Pruebas de integración
- `api.openapi.yml`: Documentación OpenAPI de la API

## Documentación API

La documentación de la API está disponible en formato OpenAPI en el archivo `api.openapi.yml`.

### Endpoints principales

- `GET /`: Información sobre la API
- `GET /books`: Obtener lista de libros (paginada)
- `POST /books`: Crear un nuevo libro (requiere autenticación)
- `GET /books/{id}`: Obtener un libro por su ID (requiere autenticación)
- `PUT /books/{id}`: Actualizar un libro (requiere autenticación)
- `DELETE /books/{id}`: Eliminar un libro (requiere autenticación)
- `GET /books/search`: Buscar libros por título o autor

### Autenticación

La API utiliza un sistema de autenticación mediante claves API. Para acceder a los endpoints protegidos, debes incluir tu clave API en el encabezado `Authorization` con el prefijo "ApiKey":

```
Authorization: ApiKey tu-clave-api-aquí
```

Puedes generar nuevas claves API utilizando la herramienta CLI incluida.


## Licencia
GPLv3
