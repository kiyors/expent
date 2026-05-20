"use client";

import type { TypedProcessedOcr } from "@expent/types";
import { toast } from "@expent/ui/components/goey-toaster";
import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useState } from "react";
import { api } from "@/lib/api-client";
import { validatePdfPageCount } from "@/lib/pdf-utils";

export type UploadStepStatus = "pending" | "in-progress" | "completed" | "failed";

export interface UploadStep {
  id: string;
  label: string;
  status: UploadStepStatus;
  description?: string;
}

export function useOcrUpload() {
  const queryClient = useQueryClient();
  const [isUploading, setIsUploading] = useState(false);
  const [uploadSteps, setUploadSteps] = useState<UploadStep[]>([]);
  const [processedOcr, setProcessedOcr] = useState<TypedProcessedOcr | null>(null);

  const pollJobStatus = async (jobId: string): Promise<TypedProcessedOcr> => {
    const maxAttempts = 150; // 5 minutes with 2s intervals
    let attempts = 0;

    while (attempts < maxAttempts) {
      const job = await api.get<any>(`/api/ocr/status/${jobId}`);

      if (job.status === "COMPLETED") {
        return job.processed_data as TypedProcessedOcr;
      }

      if (job.status === "FAILED" || job.status === "DEAD_LETTER") {
        throw new Error(job.error_message || "OCR processing failed");
      }

      if (job.status === "CONTACT_COLLISION") {
        return job.processed_data as TypedProcessedOcr;
      }

      if (job.status === "PROCESSING") {
        setUploadSteps((prev) =>
          prev.map((s) =>
            s.id === "2"
              ? { ...s, status: "completed" }
              : s.id === "3"
                ? { ...s, status: "in-progress", description: "Thinking hard..." }
                : s,
          ),
        );
      }

      await new Promise((resolve) => setTimeout(resolve, 2000));
      attempts++;
    }

    throw new Error("OCR processing timed out. It might still finish in the background.");
  };

  const uploadFile = useCallback(
    async (file: File) => {
      // PDF Page Count Validation (Client-side WASM)
      const isValid = await validatePdfPageCount(file);
      if (!isValid) return null;

      setIsUploading(true);
      setProcessedOcr(null);

      const steps: UploadStep[] = [
        { id: "1", label: "Uploading file…", status: "in-progress" },
        { id: "2", label: "Queuing document…", status: "pending" },
        { id: "3", label: "Extracting transaction data…", status: "pending" },
      ];
      setUploadSteps(steps);

      try {
        const formData = new FormData();
        formData.append("file", file);

        const uploadRes = await fetch("/api/upload", {
          method: "POST",
          body: formData,
        });

        if (!uploadRes.ok) {
          const errorBody = await uploadRes.text().catch(() => "Upload failed");
          throw new Error(errorBody || "Upload failed");
        }
        const { key } = await uploadRes.json();

        setUploadSteps((prev) =>
          prev.map((s) =>
            s.id === "1" ? { ...s, status: "completed" } : s.id === "2" ? { ...s, status: "in-progress" } : s,
          ),
        );

        const { job_id } = await api.post<any>("/api/ocr/process", { key });

        setUploadSteps((prev) =>
          prev.map((s) =>
            s.id === "2" ? { ...s, status: "completed" } : s.id === "3" ? { ...s, status: "in-progress" } : s,
          ),
        );

        const result = await pollJobStatus(job_id);

        queryClient.invalidateQueries({ queryKey: ["wallets"] });
        queryClient.invalidateQueries({ queryKey: ["contacts"] });

        setUploadSteps((prev) => prev.map((s) => (s.id === "3" ? { ...s, status: "completed" } : s)));

        setProcessedOcr(result);
        toast.success("Data extracted successfully! Please review.");
        setTimeout(() => setIsUploading(false), 1000);
        return result;
      } catch (error) {
        console.error(error);
        setUploadSteps((prev) => prev.map((s) => (s.status === "in-progress" ? { ...s, status: "failed" } : s)));
        toast.error(error instanceof Error ? error.message : "Upload or processing failed.");
        setTimeout(() => setIsUploading(false), 2000);
        return null;
      }
    },
    [queryClient.invalidateQueries, pollJobStatus],
  );

  const reset = useCallback(() => {
    setIsUploading(false);
    setUploadSteps([]);
    setProcessedOcr(null);
  }, []);

  return {
    isUploading,
    uploadSteps,
    processedOcr,
    uploadFile,
    setProcessedOcr,
    reset,
  };
}
