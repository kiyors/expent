"use client";

import { CreateCategoryDialog } from "@/components/categories/create-category-dialog";
import { GlobalOCRDialog } from "@/components/transactions/global-ocr-dialog";
import { ManualTransactionDialog } from "@/components/transactions/manual-transaction-dialog";
import { useGlobalStore } from "@/lib/store";

export function GlobalModals() {
  const {
    isTransactionModalOpen,
    setTransactionModalOpen,
    isOCRModalOpen,
    setOCRModalOpen,
    isCategoryModalOpen,
    setCategoryModalOpen,
  } = useGlobalStore();

  return (
    <>
      <ManualTransactionDialog open={isTransactionModalOpen} onOpenChange={setTransactionModalOpen} />
      <GlobalOCRDialog open={isOCRModalOpen} onOpenChange={setOCRModalOpen} />
      <CreateCategoryDialog open={isCategoryModalOpen} onOpenChange={setCategoryModalOpen} />
    </>
  );
}
