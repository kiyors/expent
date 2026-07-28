## 2026-05-11 - [Mobile Icon-Only Button Accessibility]

**Learning:** In React Native/Expo development, icon-only buttons (like Search or Filter) are completely opaque to screen readers if they lack an `accessibilityLabel`. Unlike web where `sr-only` text is a common pattern, mobile requires the explicit `accessibilityLabel` prop on the `Pressable` or `TouchableOpacity` component.
**Action:** Always audit mobile screens for `size="icon"` buttons and ensure they have a descriptive `accessibilityLabel` that conveys intent (e.g., "Filter activity" instead of just "Filter").
