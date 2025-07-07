# Rust Web-Based File Manager

A modern, secure, and fast web-based file manager built with Rust. Access and manage your files from any device with a web browser.

## Features

- **🚀 Fast & Efficient**: Built with Rust for maximum performance and memory safety
- **🔐 Secure**: Automatic system file filtering and protection
- **🌐 Web-Based**: Access files from any device with a web browser
- **📤 File Upload**: Drag-and-drop file uploads with progress tracking
- **👁️ File Viewing**: View images, videos, PDFs, and text files directly in browser
- **⬇️ Download Options**: Choose to view in browser or download files
- **🎨 Modern UI**: Clean, responsive design with glass-morphism effects
- **📱 Mobile Friendly**: Works on desktop, tablet, and mobile devices

## Screenshots

The application features a modern gradient background with glass-morphism effects:
- **Home Page**: Clean welcome screen with navigation
- **File Browser**: Grid-based file and folder view with type-specific icons
- **Upload Interface**: Drag-and-drop upload with progress tracking

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo package manager

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/rust-web-file-manager.git
cd rust-web-file-manager
```

2. Build the project:
```bash
cargo build --release
```

3. Run the server:
```bash
cargo run
```

4. Open your web browser and navigate to:
```
http://localhost:3000
```

### Dependencies

The project uses the following main dependencies:
- `hyper` - HTTP server implementation
- `tokio` - Async runtime
- `mime_guess` - MIME type detection
- `serde` - Serialization framework

## Usage

### Browsing Files

1. Click "Browse Files" from the home page
2. Navigate through folders by clicking on them
3. Use the "Parent Directory" button to go up one level
4. Files are displayed with type-specific icons and colors:
   - 📁 Orange folders
   - 📄 Blue generic files
   - 🖼️ Green image files
   - 🎥 Purple video files
   - 📝 Yellow text files
   - 📄 Red PDF files

### Viewing Files

- **View in Browser**: Click the "View" button for supported file types
- **Download**: Click the "Download" button to save files locally
- **Supported View Types**:
  - Images: JPG, PNG, GIF, BMP, WebP, SVG
  - Videos: MP4, WebM, OGG, MOV
  - Text: TXT, HTML, CSS, JS, JSON, XML, MD, CSV, LOG
  - Code: RS, PY
  - Documents: PDF

### Uploading Files

1. Click "Upload Files" from any page
2. Drag and drop files onto the upload zone, or click to browse
3. Selected files will be listed with their sizes
4. Click "Upload File(s)" to start the upload
5. Monitor progress with the animated progress bar

## Security Features

The application automatically filters and blocks access to:

### Windows System Files
- System Volume Information
- $Recycle.Bin
- Windows system directories (Windows, Program Files, etc.)
- System files (pagefile.sys, hiberfil.sys, etc.)
- Hidden and system files

### Cross-Platform Protection
- Hidden files starting with `.`
- Temporary files (`.tmp`, `.temp`)
- System configuration files
- Windows shortcuts (`.lnk`)

## Configuration

### Port Configuration
The server runs on port 8000 by default. To change the port, modify the `main.rs` file:

```rust
let addr = SocketAddr::from(([0, 0, 0, 0], 8000)); // Change 3000 to your desired port
```

### Home Directory
The application serves files from the user's home directory by default. This is determined by:
- Windows: `%USERPROFILE%`
- Unix/Linux: `$HOME`
- Fallback: Current directory

## Architecture

The project is structured as follows:

```
src/
├── main.rs          # Server setup and main loop
├── handlers.rs      # HTTP request handlers
├── file_browser.rs  # File listing and HTML generation
├── upload.rs        # File upload handling
├── utils.rs         # Utility functions
└── bodies.rs        # HTTP body implementations
```

### Key Components

- **Server**: Hyper-based HTTP server with async request handling
- **File Browser**: Secure directory traversal with system file filtering
- **Upload Handler**: Multipart form data processing
- **Security Layer**: Comprehensive system file protection

## API Endpoints

- `GET /` - Home page
- `GET /browse[/path]` - File browser interface
- `GET /file/[path]` - File serving (view or download)
- `GET /upload` - Upload interface
- `POST /upload` - File upload handler

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with the Rust programming language
- Uses Hyper for HTTP server implementation
- Inspired by modern web file managers
- Glass-morphism design trends

## Troubleshooting

### Common Issues

1. **Permission Denied**: Make sure you have read permissions for the directories you're trying to access
2. **Port Already in Use**: Change the port in `main.rs` if port 3000 is already occupied
3. **Upload Failures**: Check disk space and file permissions in the target directory

### Browser Compatibility

The application works best with modern browsers that support:
- CSS Grid
- Flexbox
- Backdrop-filter (for glass-morphism effects)
- Modern JavaScript (ES6+)

Tested on:
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Performance

- **Memory Usage**: Efficient memory management with Rust's ownership system
- **Concurrent Requests**: Handles multiple simultaneous file operations
- **Large Files**: Streaming support for large file downloads
- **Responsive UI**: Smooth animations and transitions

---

**Note**: This file manager is designed for personal use and local networks. For production deployment, consider additional security measures such as authentication, HTTPS, and access controls.

