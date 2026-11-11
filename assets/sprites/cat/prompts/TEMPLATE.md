# Canon — Desktop Cat sprites (REALISTIC)

## Exact strip size (как заказывать)

Для N кадров по 128×128:

- **height** = `128`
- **width** = `128 * N`
- В промпте явно: «exactly N equal square panels, each 128x128»
- Каждый panel — отдельная поза (лапы перечислить)

Генерация: `npm run sprites:strip -- walk` → Classic API с точным `width`/`height`  
(не Cursor GenerateImage — там только фиксированные aspect ratio).

## Style (LOCK)

- Soft realistic cutout по фото, **не** pixel art.
- Tortie, split-face, white bib/socks, amber eyes.
- Фон `#00FF00`, все мастера **RIGHT**; влево — flip в коде.

## Scale

После скачивания: slice → height-lock `body_h=112` в ячейку `200×128` (бег не сжимаем).
