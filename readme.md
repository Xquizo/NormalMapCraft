# Normal Map Pro (Multi-Scale) — PhotoCraft Plugin

Un plugin nativo compilado a **WebAssembly (`wasm32-unknown-unknown`)** escrito en Rust puro (`#![no_std]`) para generar **Normal Maps** de alta calidad directamente dentro de **PhotoCraft**, optimizado especialmente para texturas de vegetación (*foliage*), cortezas, materiales orgánicos y atlas PBR.

---

## ✨ Características

* **Operador Sobel Multi-Escala (Pirámide de Relieve):** Combina detalle fino (`1 px`), volumen medio y bisel ancho (`bevel`) en una sola pasada para lograr tanto micro-textura nítida como volumen redondeado sin perder definición.
* **Orientación Direccional Completa:**
  * `OpenGL (+Y / Unity)` — Estándar para Unity, Godot, Blender y Unreal (con flip Y).
  * `Sur (3ds Max / DirectX)` — Canal verde invertido (`-Y`).
  * `Este` / `Oeste` — Rotación lateral de incidencia de relieve ($\pm 90^\circ$).
* **Relleno Inteligente de Transparencia (`#8080FF`):** Rellena las zonas transparentes con el color normal neutro `RGB(128, 128, 255)` y neutraliza la altura en los bordes del recorte alfa, evitando halos oscuros o artefactos al generar *mipmaps* en motores 3D.
* **Pre-Blur y Contraste de Altura:** Suavizado previo opcional para eliminar ruido fotográfico antes de calcular pendientes y curva de contraste para acentuar grietas profundas.
* **Procesamiento sin Costuras entre Cuadrantes:** Configurado con `"overlap": 12` y `"area": "canvas"` para procesar documentos completos divididos en *tiles* sin líneas de unión.
* **Cero Dependencias (`#![no_std]`):** Binario ultraligero (~3 KB) sin asignaciones dinámicas en tiempo de ejecución ni colisiones de memoria en el *sandbox* WASM.

---

## 🎛️ Parámetros del Filtro

| Parámetro | Opciones | Descripción |
| :--- | :--- | :--- |
| **Detail Scale** | `Bajo (1.5)`, `Medio (3.5)`, `Alto (6.0)`, `Muy Alto (10.0)`, `Extremo (16.0)` | Intensidad general de la pendiente del relieve. |
| **Bevel** | `Fino (1 px)`, `Medio (3 px)`, `Ancho (6 px)`, `Volumetrico (10 px)` | Radio de muestreo multi-escala para controlar qué tan ancho y redondeado es el bisel de las normales. |
| **Dirección** | `Seleccionar...`, `OpenGL (+Y / Unity)`, `Sur (3ds Max / DirectX)`, `Este`, `Oeste` | Convención de ejes del Normal Map. *(Nota: inicia en `Seleccionar...` para activar el renderizado de lienzo completo al elegir la orientación).* |
| **Desenfocar** | `0 (Ninguno)`, `1 px (Sutil)`, `2 px (Suave)` | Suavizado previo del mapa de luminancia antes de calcular el operador Sobel. |
| **Contraste** | `Lineal (1.0)`, `Medio (1.3)`, `Fuerte (1.7)` | Curva de contraste aplicada a la altura antes de extraer normales. |
| **Transparencia** | `Rellenar neutro (#8080FF)`, `Conservar alfa` | Rellena el fondo transparente con normal plana `(0.5, 0.5, 1.0)` para proteger los *mipmaps* o conserva el canal alfa original. |

---

## 🛠️ Requisitos e Instalación del Entorno (Rust + WASM)

Este proyecto no utiliza *crates* externos; únicamente requiere el compilador de Rust y el *target* de WebAssembly.

### 1. Instalar Rust y Cargo
Si aún no tienes Rust instalado en tu sistema:

* **Windows (PowerShell):**
  Descarga e instala desde [rustup.rs](https://rustup.rs/) o ejecuta:
  ```powershell
  winget install Rustlang.Rustup
  ```
* **Linux / macOS:**
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### 2. Agregar el target `wasm32-unknown-unknown`
Una vez instalado Rust, reinicia tu terminal y añade el soporte para compilar a WebAssembly sin sistema operativo:

```bash
rustup target add wasm32-unknown-unknown
```

---

## ⚙️ Compilación

1. Clona este repositorio y entra en la carpeta del proyecto:
   ```bash
   git clone <URL_DE_TU_REPOSITORIO>
   cd NormalMap
   ```

2. Compila el binario optimizado en modo `release`:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

3. El archivo compilado se generará en:
   ```text
   target/wasm32-unknown-unknown/release/photocraft_normal_map.wasm
   ```

### Configuración recomendada en `Cargo.toml`
Para mantener el binario mínimo y compatible con `no_std`, asegúrate de conservar estas opciones en tu `Cargo.toml`:

```toml
[package]
name = "photocraft_normal_map"
version = "1.4.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = 3
lto = true
strip = true
panic = "abort"
```

---

## 📥 Instalación en PhotoCraft

1. Abre **PhotoCraft**.
2. Ve al menú superior: **Filter > Plug-in > Instalar...**
3. Selecciona el archivo `photocraft_normal_map.wasm` generado en la carpeta `target/wasm32-unknown-unknown/release/`.
4. Abre cualquier imagen o capa y ejecuta **Filter > Normal Map Pro (Multi-Scale)**.
5. En el desplegable **Dirección**, selecciona **OpenGL (+Y / Unity)** (o la orientación deseada) para generar el mapa sobre todo el lienzo.

---

## 📄 Licencia

Distribuido bajo licencia MIT. Libre para usar, modificar y compartir en proyectos personales o comerciales.