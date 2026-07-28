# Expent Mobile App (`apps/app`)

The Expent mobile application is a universal app built with **Expo** and **React Native**, designed for on-the-go expense tracking, OCR capture, and quick P2P approvals.

## 1. Tech Stack

- **Framework**: [Expo](https://expo.dev/) with [Expo Router](https://docs.expo.dev/router/introduction/) (File-based routing).
- **UI Components**: [React Native Reusables](https://rnr-docs.vercel.app/) (Radix UI equivalent for React Native).
- **Styling**: [NativeWind v4](https://www.nativewind.dev/) (Tailwind CSS for React Native).
- **Data Fetching**: [TanStack Query (React Query)](https://tanstack.com/query/latest) with [Axios](https://axios-http.com/).
- **State Management**: [Zustand](https://docs.pmnd.rs/zustand/getting-started/introduction).
- **Lists**: [@shopify/flash-list](https://shopify.github.io/flash-list/) for high-performance scrolling.
- **Animations**: [Moti](https://moti.fyi/) (powered by Reanimated).

---

## 2. Directory Structure

- **`src/app/`**: Contains the routes and screen definitions.
  - **`(auth)/`**: Login and Sign-up screens.
  - **`(tabs)/`**: Main application navigation (Dashboard, Activity, Wallets, etc.).
- **`src/components/`**: Reusable UI blocks and business-specific components (e.g., `TransactionCard`).
- **`src/lib/`**: Essential infrastructure logic.
  - **`api/`**: Axios client configuration and TanStack Query providers.
  - **`auth/`**: Authentication hooks and session management logic.
  - **`storage.ts`**: Wrapper for `expo-secure-store` or `AsyncStorage`.
  - **`theme.ts`**: Theming configuration for light/dark mode.

---

## 3. Key Patterns

### Data Fetching

The app uses a central Axios client configured with the `EXPO_PUBLIC_API_URL`. All requests include the `better-auth` session token retrieved from secure storage.

### Authentication

Authentication is bridged to the Rust backend's `better-auth` implementation. The `useAuth` hook manages the user state and persists sessions using secure device storage.

### Responsive Design

Using NativeWind, the app shares the same Tailwind utility-first approach as the dashboard, ensuring visual consistency. It uses a "Mobile-First" strategy, optimized for both iOS and Android.

---

## 4. Development Workflow

### Running the App

Ensure you have the [Expo Go](https://expo.dev/go) app installed on your physical device or an emulator set up.

```bash
# Start the Expo development server
pnpm dev:app
```

### Environment Variables

The app requires an `env.ts` or `.env` file containing:

- `EXPO_PUBLIC_API_URL`: The URL of the `apps/api` gateway.
- `EXPO_PUBLIC_APP_ENV`: `development` or `production`.
