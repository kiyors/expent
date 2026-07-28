import { create } from "zustand";

interface GlobalStore {
  isTransactionModalOpen: boolean;
  setTransactionModalOpen: (open: boolean) => void;
  isOCRModalOpen: boolean;
  setOCRModalOpen: (open: boolean) => void;
  isCategoryModalOpen: boolean;
  setCategoryModalOpen: (open: boolean) => void;
  isHotkeyHelpOpen: boolean;
  setHotkeyHelpOpen: (open: boolean) => void;
}

export const useGlobalStore = create<GlobalStore>((set) => ({
  isTransactionModalOpen: false,
  setTransactionModalOpen: (open) => set({ isTransactionModalOpen: open }),
  isOCRModalOpen: false,
  setOCRModalOpen: (open) => set({ isOCRModalOpen: open }),
  isCategoryModalOpen: false,
  setCategoryModalOpen: (open) => set({ isCategoryModalOpen: open }),
  isHotkeyHelpOpen: false,
  setHotkeyHelpOpen: (open) => set({ isHotkeyHelpOpen: open }),
}));
