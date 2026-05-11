import { router } from "expo-router";
import * as React from "react";
import { View } from "react-native";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Text } from "@/components/ui/text";
import { showErrorMessage } from "@/components/ui/utils";
import { useAuth } from "@/lib/auth/use-auth";

export function ForgotPasswordForm() {
  const [email, setEmail] = React.useState("");
  const { forgotPassword, isLoading } = useAuth();

  async function onSubmit() {
    if (!email) {
      showErrorMessage("Please enter your email");
      return;
    }

    try {
      await forgotPassword(email);
      router.push("/(auth)/reset-password");
    } catch (error) {
      showErrorMessage(error instanceof Error ? error.message : "Something went wrong");
    }
  }

  return (
    <View className="gap-6">
      <Card className="border-border/0 sm:border-border shadow-none sm:shadow-sm sm:shadow-black/5">
        <CardHeader>
          <CardTitle className="text-center text-xl sm:text-left">Forgot password?</CardTitle>
          <CardDescription className="text-center sm:text-left">
            Enter your email to reset your password
          </CardDescription>
        </CardHeader>
        <CardContent className="gap-6">
          <View className="gap-6">
            <View className="gap-1.5">
              <Label nativeID="email-label">Email</Label>
              <Input
                id="email"
                placeholder="m@example.com"
                keyboardType="email-address"
                autoComplete="email"
                autoCapitalize="none"
                returnKeyType="send"
                value={email}
                onChangeText={setEmail}
                onSubmitEditing={onSubmit}
                aria-labelledby="email-label"
              />
            </View>
            <Button className="w-full" onPress={onSubmit} disabled={isLoading}>
              <Text>{isLoading ? "Sending..." : "Reset your password"}</Text>
            </Button>
            <Button variant="ghost" className="w-full" onPress={() => router.back()}>
              <Text>Back to sign in</Text>
            </Button>
          </View>
        </CardContent>
      </Card>
    </View>
  );
}
