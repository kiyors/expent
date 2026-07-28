import { CreateCategoryDialog } from "@/components/categories/CreateCategoryDialog";
import { GlobalOCRDialog } from "@/components/transactions/GlobalOcrDialog";
import { ManualTransactionDialog } from "@/components/transactions/ManualTransactionDialog";
import { useGlobalStore } from "@/lib/Store";

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
