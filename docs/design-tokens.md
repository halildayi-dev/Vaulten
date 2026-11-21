# Vaulten Design Tokens

## 1. Color

### Light Theme

```css
--color-background: #F8FAFC;
--color-surface: #FFFFFF;
--color-surface-muted: #F1F5F9;

--color-border: #E2E8F0;
--color-border-strong: #CBD5E1;

--color-text-primary: #0F172A;
--color-text-secondary: #475569;
--color-text-muted: #64748B;
--color-text-disabled: #94A3B8;

--color-primary: #4F46E5;
--color-primary-hover: #4338CA;
--color-primary-active: #3730A3;
--color-primary-foreground: #FFFFFF;

--color-danger: #DC2626;
--color-danger-hover: #B91C1C;
--color-danger-foreground: #FFFFFF;

--color-success: #16A34A;
--color-warning: #D97706;

--color-focus: #6366F1;
```

### Dark Theme

```css
--color-background: #0F172A;
--color-surface: #111827;
--color-surface-muted: #1E293B;

--color-border: #334155;
--color-border-strong: #475569;

--color-text-primary: #F8FAFC;
--color-text-secondary: #CBD5E1;
--color-text-muted: #94A3B8;
--color-text-disabled: #64748B;

--color-primary: #6366F1;
--color-primary-hover: #818CF8;
--color-primary-active: #4F46E5;
--color-primary-foreground: #FFFFFF;

--color-danger: #EF4444;
--color-danger-hover: #F87171;
--color-danger-foreground: #FFFFFF;

--color-success: #22C55E;
--color-warning: #F59E0B;

--color-focus: #818CF8;
```

### Color Rules

- Use semantic tokens instead of raw colors.
- Primary color is reserved for primary actions and important interactive states.
- Danger colors are reserved for destructive actions and destructive feedback.
- Do not introduce additional accent colors without a defined semantic purpose.
- Avoid gradients.

---

## 2. Typography

### Font Family

```css
--font-family-sans: "Inter", system-ui, -apple-system, BlinkMacSystemFont,
  "Segoe UI", sans-serif;
```

### Font Sizes

```css
--font-size-xs: 0.75rem;     /* 12px */
--font-size-sm: 0.875rem;    /* 14px */
--font-size-md: 1rem;        /* 16px */
--font-size-lg: 1.125rem;    /* 18px */
--font-size-xl: 1.25rem;     /* 20px */
--font-size-2xl: 1.5rem;     /* 24px */
```

### Font Weights

```css
--font-weight-regular: 400;
--font-weight-medium: 500;
--font-weight-semibold: 600;
--font-weight-bold: 700;
```

### Line Heights

```css
--line-height-tight: 1.25;
--line-height-normal: 1.5;
--line-height-relaxed: 1.625;
```

### Typography Usage

- Body text: `16px / 400`
- Secondary text: `14px / 400`
- Labels: `14px / 500`
- Small metadata: `12px / 400`
- Section titles: `18px / 600`
- Page titles: `24px / 600`

---

## 3. Spacing

Use a 4px base spacing system.

```css
--spacing-1: 4px;
--spacing-2: 8px;
--spacing-3: 12px;
--spacing-4: 16px;
--spacing-5: 20px;
--spacing-6: 24px;
--spacing-8: 32px;
--spacing-10: 40px;
--spacing-12: 48px;
```

---

## 4. Border Radius

```css
--radius-sm: 6px;
--radius-md: 8px;
--radius-lg: 10px;
--radius-xl: 12px;
```

### Radius Usage

- Small controls: `6px`
- Inputs and buttons: `8px`
- Cards: `10px`
- Dialogs/modals: `12px`

Avoid excessive rounded or pill-shaped components.

---

## 5. Border

```css
--border-width-default: 1px;
--border-width-strong: 2px;
```

Use `1px` borders by default.

Use `2px` only for strong focus or emphasis states.

---

## 6. Shadows

Keep shadows subtle.

```css
--shadow-sm: 0 1px 2px rgba(15, 23, 42, 0.05);
--shadow-md: 0 4px 12px rgba(15, 23, 42, 0.08);
--shadow-lg: 0 10px 24px rgba(15, 23, 42, 0.10);
```

Use shadows sparingly. Prefer borders and surface contrast for hierarchy.

---

## 7. Component Dimensions

### Buttons

```css
--button-height-sm: 32px;
--button-height-md: 36px;
--button-height-lg: 40px;

--button-padding-x-sm: 12px;
--button-padding-x-md: 16px;
--button-padding-x-lg: 20px;
```

### Inputs

```css
--input-height-sm: 32px;
--input-height-md: 36px;
--input-height-lg: 40px;
```

Default input height: `36px`.

### Icon Buttons

```css
--icon-button-size-sm: 28px;
--icon-button-size-md: 32px;
--icon-button-size-lg: 36px;
```

Default icon button size: `32px`.

### Icons

```css
--icon-size-sm: 14px;
--icon-size-md: 16px;
--icon-size-lg: 20px;
```

Default UI icon size: `16px`.

---

## 8. Layout Dimensions

```css
--sidebar-width: 240px;
--sidebar-width-collapsed: 64px;

--header-height: 56px;

--content-max-width: 1200px;
```

---

## 9. Interaction States

### Focus

```css
--focus-ring-width: 2px;
--focus-ring-offset: 2px;
```

Focus states must be clearly visible.

### Disabled

Use:

```css
opacity: 0.5;
```

Disabled controls must not appear interactive.

### Hover

Hover states should use subtle surface or border changes rather than dramatic color changes.

### Active / Selected

Selected navigation items should use a subtle background and primary text/icon color.

---

## 10. Iconography

- Use functional icons only.
- Default icon size: `16px`.
- Use `20px` icons for prominent actions.
- Icons must have consistent stroke weight and visual style.
- Do not use decorative illustrations as UI elements.
- Do not use oversized icons for ordinary controls.
- Icon-only buttons must have an accessible label.

---

## 11. Surface Hierarchy

Use the following hierarchy:

```text
Background
  ↓
Surface
  ↓
Surface Muted
  ↓
Border / Border Strong
```

Avoid excessive nested cards.

Use spacing, borders, and surface contrast before adding shadows.

---

## 12. Density

Vaulten uses a compact desktop UI.

Default vertical spacing:

```text
Component-to-component: 12–16px
Section spacing:         24–32px
Page padding:            24–32px
```

Controls should remain compact and information-dense.

Avoid oversized cards, excessive empty space, and unnecessarily tall components.