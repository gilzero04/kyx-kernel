# Kyx Platform

### ระบบแพลตฟอร์มอัจฉริยะสำหรับธุรกิจยุคใหม่

---

## ภาพรวม

**Kyx Platform** คือโครงสร้างพื้นฐานซอฟต์แวร์ระดับองค์กรที่ออกแบบมาเพื่อรองรับธุรกิจหลากหลายรูปแบบ ตั้งแต่ร้านค้าออนไลน์ไปจนถึงตลาดกลางสินค้า (Marketplace)

---

## จุดเด่นหลัก

### 1. สถาปัตยกรรมหลายระดับผู้เช่า (Multi-tier Multi-tenancy)

- รองรับโครงสร้างองค์กรแบบลำดับชั้น เช่น เจ้าของแพลตฟอร์ม → สาขา → ผู้ขาย → พันธมิตร
- แต่ละระดับสามารถปรับแต่งแบรนด์และการตั้งค่าได้อิสระ
- รองรับ Custom Domain ต่อผู้เช่า

### 2. พร้อมสำหรับการเงินตั้งแต่วันแรก (Financial-Ready Architecture)

- ออกแบบตามมาตรฐาน ACID สำหรับธุรกรรมทางการเงิน
- แยกระบบ Ledger อย่างชัดเจน รองรับการตรวจสอบบัญชี
- PostgreSQL เป็น Source of Truth สำหรับข้อมูลทางการเงิน

### 3. ระบบเรียลไทม์แบบ Native

- รองรับแชทสด (LiveChat) พร้อม message ordering ต่อห้อง
- วิดีโอคอลผ่าน WebRTC/LiveKit
- ระบบ Presence แบบ TTL-based
- รองรับการเชื่อมต่อพร้อมกันจำนวนมากพร้อม graceful degradation

### 4. ระบบปลั๊กอิน (Plugin Architecture)

- แกนกลางระบบเบาและเสถียร
- เพิ่มฟีเจอร์ผ่านปลั๊กอิน WASM-based
- ตัวอย่างปลั๊กอิน: กระเป๋าเงิน, ระบบชำระเงิน, วิเคราะห์ข้อมูล
- Capability-based Security สำหรับแต่ละปลั๊กอิน

### 5. ระบบธรรมาภิบาล AI (AI Governance)

- กฎควบคุม 32 ข้อสำหรับพฤติกรรม AI
- ป้องกัน AI ทำผิดพลาดในระบบ Production
- Knowledge Base พร้อม Semantic Search
- Incident Tracking สำหรับเรียนรู้จากปัญหาที่ผ่านมา

---

## เทคโนโลยีหลัก

| ส่วนประกอบ         | เทคโนโลยี                | หน้าที่                            |
| ------------------ | ------------------------ | ---------------------------------- |
| **kyx-kernel**     | Rust + Ntex + PostgreSQL | แกนกลาง, Auth, RBAC, Multi-tenancy |
| **kyx-signal**     | Rust + Ntex + SurrealDB  | Realtime, Chat, Video Signaling    |
| **kyx-platform**   | SvelteKit + TypeScript   | Frontend UI                        |
| **kyx-governance** | Rust + SurrealDB         | AI Knowledge Base, Rules           |
| **kyx-infra**      | Redis + Docker + TLS     | Cache, Session, Deployment         |

---

## ฟีเจอร์ระดับองค์กร

| ฟีเจอร์                    | สถานะ          |
| -------------------------- | -------------- |
| Hierarchical Tenants       | ✅ พร้อมใช้งาน |
| Custom Branding ต่อ Tenant | ✅ พร้อมใช้งาน |
| Role-based Landing Pages   | ✅ พร้อมใช้งาน |
| API Key Management         | ✅ พร้อมใช้งาน |
| Audit Logging              | ✅ พร้อมใช้งาน |
| i18n (ไทย/อังกฤษ)          | ✅ พร้อมใช้งาน |
| Theme System               | ✅ พร้อมใช้งาน |
| Plugin System              | 🔄 กำลังพัฒนา  |
| Payment Gateway            | 📋 ในแผน       |

---

## เหมาะสำหรับ

### ธุรกิจค้าปลีก

- **E-commerce & Marketplace**: แพลตฟอร์มหลายผู้ขายที่มีโครงสร้างซับซ้อน
- **POS (Point of Sale)**: ระบบขายหน้าร้านพร้อม inventory และ multi-branch

### ธุรกิจบริการ

- **ระบบบริการโรงพยาบาล**: นัดหมาย, ประวัติผู้ป่วย, การแจ้งเตือน real-time
- **HR Management**: จัดการพนักงาน, โครงสร้างองค์กร, การลา

### ธุรกิจการเงิน

- **ระบบการเงิน/Fintech**: กระเป๋าเงิน, การโอน, รายงานทางการเงิน
- **SaaS Applications**: โซลูชัน White-label ที่ต้องการแยก Tenant

### แพลตฟอร์มสื่อสาร

- **Communication Platforms**: ระบบแชทและวิดีโอคอลในตัว
- **AI-Powered Services**: บูรณาการ AI อย่างปลอดภัยด้วย Governance

---

## ข้อได้เปรียบทางการแข่งขัน

| เทียบกับ     | Kyx ดีกว่าตรงไหน                          |
| ------------ | ----------------------------------------- |
| Shopify      | Self-hosted, Multi-tier tenancy แท้จริง   |
| Firebase     | PostgreSQL ACID, Financial compliance     |
| Supabase     | ระบบ Realtime-native, Plugin architecture |
| Custom Build | 80% พร้อมใช้, Governance ครบ              |

---

## การติดต่อ

สำหรับข้อมูลเพิ่มเติมหรือการสาธิตระบบ กรุณาติดต่อทีมพัฒนา

---

_เอกสารนี้เป็นส่วนหนึ่งของ Kyx Platform Documentation_
_อัปเดตล่าสุด: มกราคม 2026_
