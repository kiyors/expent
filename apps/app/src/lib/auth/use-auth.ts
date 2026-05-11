import type { User } from "@expent/types";
import { useEffect, useState } from "react";
import { create } from "zustand";
import { client } from "@/lib/api";
import { getItem, removeItem, setItem } from "@/lib/storage";

const AUTH_KEY = "auth_session";

interface AuthState {
  isAuthenticated: boolean;
  isLoading: boolean;
  user: User | null;
  setAuth: (user: User) => void;
  clearAuth: () => void;
  setLoading: (loading: boolean) => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  isAuthenticated: false,
  isLoading: true, // Start loading to check storage
  user: null,
  setAuth: (user: User) => {
    setItem(AUTH_KEY, user);
    set({ isAuthenticated: true, user, isLoading: false });
  },
  clearAuth: () => {
    removeItem(AUTH_KEY);
    set({ isAuthenticated: false, user: null, isLoading: false });
  },
  setLoading: (loading: boolean) => set({ isLoading: loading }),
}));

export function useAuth() {
  const { isAuthenticated, isLoading, user, setAuth, clearAuth } = useAuthStore();
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Check storage on mount
  useEffect(() => {
    const checkSession = async () => {
      try {
        const storedUser = await getItem(AUTH_KEY);
        if (storedUser) {
          setAuth(storedUser as User);
        } else {
          clearAuth();
        }
      } catch (_error) {
        clearAuth();
      }
    };
    checkSession();
  }, [setAuth, clearAuth]);

  const signIn = async (email?: string, password?: string) => {
    setIsSubmitting(true);
    try {
      // Temporary stub for Better Auth integration
      console.log("Signing in with:", email);
      const response = await client.post("/auth/sign-in/email", {
        email,
        password,
      });
      setAuth(response.data.user || { email });
      return response.data;
    } catch (error) {
      console.error(error);
      throw error;
    } finally {
      setIsSubmitting(false);
    }
  };

  const signUp = async (email?: string, password?: string, name?: string) => {
    setIsSubmitting(true);
    try {
      console.log("Signing up with:", email, name);
      // Temporary stub for Better Auth integration
      const response = await client.post("/auth/sign-up/email", {
        email,
        password,
        name,
      });
      setAuth(response.data.user || { email, name });
      return response.data;
    } catch (error) {
      console.error(error);
      throw error;
    } finally {
      setIsSubmitting(false);
    }
  };

  const signOut = async () => {
    setIsSubmitting(true);
    try {
      await client.post("/auth/sign-out");
    } catch (error) {
      console.error(error);
    } finally {
      clearAuth();
      setIsSubmitting(false);
    }
  };

  const forgotPassword = async (email: string) => {
    setIsSubmitting(true);
    try {
      const response = await client.post("/auth/forget-password", {
        email,
        redirectTo: "expent://reset-password",
      });
      return response.data;
    } catch (error) {
      console.error(error);
      throw error;
    } finally {
      setIsSubmitting(false);
    }
  };

  const resetPassword = async (newPassword: string, token: string) => {
    setIsSubmitting(true);
    try {
      const response = await client.post("/auth/reset-password", {
        newPassword,
        token,
      });
      return response.data;
    } catch (error) {
      console.error(error);
      throw error;
    } finally {
      setIsSubmitting(false);
    }
  };

  return {
    isAuthenticated,
    user,
    signIn,
    signUp,
    signOut,
    forgotPassword,
    resetPassword,
    isLoading: isLoading || isSubmitting,
    isInitialized: !isLoading,
  };
}
