"use client";

import type { TypedProcessedOcr } from "@expent/types";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@expent/ui/components/dialog";
import { toast } from "@expent/ui/components/goey-toaster";
import { Input } from "@expent/ui/components/input";
import { useQueryClient } from "@tanstack/react-query";
import { CameraIcon, Loader2Icon, SparklesIcon } from "lucide-react";
import * as React from "react";
import { ProgressTracker } from "@/components/tool-ui/progress-tracker";
import { useOcrUpload } from "@/hooks/use-ocr";
import { api } from "@/lib/api-client";
import { ReviewTransactionForm } from "./review-transaction-form";

export function GlobalOCRDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (o: boolean) => void }) {
  const { isUploading, uploadSteps, processedOcr, uploadFile, setProcessedOcr, reset } = useOcrUpload();
  const [isSaving, setIsSaving] = React.useState(false);
  const queryClient = useQueryClient();

  const handleUpload = async (selectedFile: File) => {
    await uploadFile(selectedFile);
  };

  const handleConfirm = async (finalData: TypedProcessedOcr) => {
    setIsSaving(true);
    try {
      await api.post("/api/transactions/from-ocr", finalData);
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      toast.success("Transaction saved!");
      onOpenChange(false);
      reset();
    } catch (_error) {
      toast.error("Failed to save transaction");
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(val) => {
        if (!isUploading && !isSaving) {
          onOpenChange(val);
          if (!val) {
            reset();
          }
        }
      }}
    >
      <DialogContent className={processedOcr ? "sm:max-w-3xl" : "sm:max-w-md"}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <SparklesIcon className="h-5 w-5 text-primary animate-pulse" />
            Scan Receipt
          </DialogTitle>
          <DialogDescription>
            Upload a receipt image to automatically extract transaction details using AI.
          </DialogDescription>
        </DialogHeader>

        {!processedOcr && !isUploading && (
          <div className="flex flex-col items-center justify-center border-2 border-dashed border-muted-foreground/20 rounded-xl p-12 transition-colors hover:border-primary/50 group cursor-pointer relative">
            <Input
              type="file"
              accept="image/*"
              className="absolute inset-0 opacity-0 cursor-pointer z-10"
              onChange={(e) => {
                const f = e.target.files?.[0];
                if (f) handleUpload(f);
              }}
            />
            <div className="bg-primary/5 size-16 rounded-full flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
              <CameraIcon className="h-8 w-8 text-primary" />
            </div>
            <p className="font-semibold text-foreground">Click or Drag to Upload</p>
            <p className="text-sm text-muted-foreground mt-1 text-center">Supports JPG, PNG and PDF receipts</p>
          </div>
        )}

        {isUploading && (
          <div className="py-8 space-y-6">
            <div className="flex flex-col items-center justify-center text-center">
              <Loader2Icon className="h-10 w-10 text-primary animate-spin mb-4" />
              <p className="font-medium">Magically extracting data...</p>
            </div>
            <ProgressTracker id="global-ocr-progress" steps={uploadSteps} />
          </div>
        )}

        {processedOcr && (
          <div className="mt-4">
            <ReviewTransactionForm
              processedOcr={processedOcr}
              onConfirm={handleConfirm}
              onCancel={() => onOpenChange(false)}
              isSubmitting={isSaving}
            />
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
