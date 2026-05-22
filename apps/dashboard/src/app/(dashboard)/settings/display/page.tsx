"use client";

import { Button } from "@expent/ui/components/button";
import { toast } from "@expent/ui/components/goey-toaster";
import { Label } from "@expent/ui/components/label";
import { RadioGroup, RadioGroupItem } from "@expent/ui/components/radio-group";
import { Separator } from "@expent/ui/components/separator";
import { Switch } from "@expent/ui/components/switch";

export default function SettingsDisplayPage() {
  return (
    <div className="gap-y-6 w-full max-w-2xl">
      <div>
        <h3 className="text-lg font-medium">Display</h3>
        <p className="text-sm text-muted-foreground">
          Turn items on or off to control what&apos;s displayed in the app.
        </p>
      </div>
      <Separator />
      <div className="gap-y-8">
        <div className="gap-y-4">
          <Label>Sidebar Items</Label>
          <p className="text-[0.8rem] text-muted-foreground">Select the items you want to display in the sidebar.</p>
          <div className="gap-y-3">
            {[
              { label: "Transactions", defaultChecked: true },
              { label: "Contacts", defaultChecked: true },
              { label: "Wallets", defaultChecked: true },
              { label: "Subscriptions", defaultChecked: true },
              { label: "P2P Transfers", defaultChecked: false },
              { label: "Reconciliation", defaultChecked: false },
            ].map((item) => (
              <div key={item.label} className="flex items-center justify-between rounded-lg border p-3">
                <Label className="font-normal">{item.label}</Label>
                <Switch defaultChecked={item.defaultChecked} />
              </div>
            ))}
          </div>
        </div>

        <div className="gap-y-4">
          <Label>Date Format</Label>
          <RadioGroup defaultValue="relative" className="grid gap-3">
            <div className="flex items-center gap-x-3">
              <RadioGroupItem value="relative" id="date-relative" />
              <Label htmlFor="date-relative" className="font-normal">
                Relative (e.g. &quot;2 days ago&quot;)
              </Label>
            </div>
            <div className="flex items-center gap-x-3">
              <RadioGroupItem value="absolute" id="date-absolute" />
              <Label htmlFor="date-absolute" className="font-normal">
                Absolute (e.g. &quot;Apr 5, 2026&quot;)
              </Label>
            </div>
          </RadioGroup>
        </div>

        <Button onClick={() => toast.success("Display settings saved!")}>Update display</Button>
      </div>
    </div>
  );
}
