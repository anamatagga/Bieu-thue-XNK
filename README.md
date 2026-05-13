# 📊 Biểu Thuế Xuất Nhập Khẩu 2026

Phần mềm tra cứu Biểu thuế XNK 2026 dành cho Windows 10/11 64-bit.  
Giao diện **Hyprdots style** (Catppuccin Mocha), viết bằng **Rust + egui**.

---

## ✨ Tính năng

| Tính năng | Chi tiết |
|-----------|---------|
| 🔍 Tìm theo mã HS | Nhập 4, 6 hoặc 8 số — hiện toàn bộ nhóm |
| 🔍 Tìm theo tên hàng | Tiếng Việt (có dấu hoặc không dấu) và tiếng Anh |
| 📋 Dữ liệu đầy đủ | 15,119 mã HS nhúng trực tiếp trong .exe |
| 🌏 Chế độ FTA | 19 hiệp định: ACFTA, ATIGA, CPTPP, EVFTA, UKVFTA... |
| 📤 Chế độ xuất khẩu | TT ĐB, Thuế XK, BVMT |
| 🎨 Hyprdots UI | Catppuccin Mocha, rounded corners, màu thuế rate |
| ⌨️ Phím tắt | Enter = tìm · Esc = xoá · Ctrl+F = focus |
| 📦 Standalone | 1 file .exe duy nhất, không cần cài thêm |

---

## 🛠️ Build từ source

### Yêu cầu
- Windows 10/11 64-bit
- [Rust](https://rustup.rs/) stable (≥ 1.80)

### Bước thực hiện

```powershell
# Clone hoặc giải nén project
cd bieu_thue_xnk

# Build release (~5-10 phút lần đầu, tải dependencies qua internet)
cargo build --release

# File exe nằm tại:
.\target\release\bieu-thue-xnk.exe
```

> **Lưu ý:** File exe sau khi build sẽ ~20-30MB và chứa toàn bộ dữ liệu,  
> không cần đặt kèm file JSON hay file phụ nào khác.

### Build nhanh (debug, không tối ưu)
```powershell
cargo run
```

---

## 🖥️ Build tự động qua GitHub Actions

Đặt toàn bộ project lên GitHub, workflow `.github/workflows/build.yml`  
sẽ tự động build `.exe` cho Windows 64-bit khi push lên.

---

## 📐 Cấu trúc project

```
bieu_thue_xnk/
├── Cargo.toml              ← Dependencies (eframe 0.29, serde_json)
├── src/
│   ├── main.rs             ← Entry point, window setup, icon
│   ├── app.rs              ← Toàn bộ UI logic (search, table, groups)
│   ├── theme.rs            ← Catppuccin Mocha palette + egui Visuals
│   └── data.rs             ← HsEntry struct, loader, normalize VN
└── assets/
    └── hs_data.json        ← 15,119 mã HS (nhúng vào binary lúc compile)
```

---

## 🎨 Màu sắc thuế suất

| Màu | Ý nghĩa |
|-----|---------|
| 🟢 Xanh lá | 0 – 5% |
| 🟡 Vàng | 6 – 15% |
| 🟠 Cam | 16 – 30% |
| 🔴 Đỏ | > 30% |
| ⚫ Xám | 0% |
| 🔵 Xanh dương | Đặc biệt (*) |

---

## 📄 Nguồn dữ liệu

Bộ Tài chính Việt Nam — Biểu thuế XNK 2026, cập nhật 05/04/2026.
