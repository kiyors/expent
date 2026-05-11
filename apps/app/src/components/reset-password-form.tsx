import { router } from "expo-router";
import * as React from "react";
import { type TextInput, View } from "react-native";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Text } from "@/components/ui/text";
import { showErrorMessage } from "@/components/ui/utils";
import { useAuth } from "@/lib/auth/use-auth";

export function ResetPasswordForm() {
  const [password, setPassword] = React.useState("");
  const [code, setCode] = React.useState("");
  const { resetPassword, isLoading } = useAuth();
  const codeInputRef = React.useRef<TextInput>(null);

  function onPasswordSubmitEditing() {
    codeInputRef.current?.focus();
  }

  async function onSubmit() {
    if (!password || !code) {
      showErrorMessage("Please fill in all fields");
      return;
    }

    try {
      await resetPassword(password, code);
      router.replace("/(auth)/sign-in");
    } catch (error) {
      showErrorMessage(error instanceof Error ? error.message : "Something went wrong");
    }
  }

  return (
    <View className="gap-6">
      <Card className="border-border/0 sm:border-border shadow-none sm:shadow-sm sm:shadow-black/5">
        <CardHeader>
          <CardTitle className="text-center text-xl sm:text-left">Reset password</CardTitle>
          <CardDescription className="text-center sm:text-left">
            Enter the code sent to your email and set a new password
          </CardDescription>
        </CardHeader>
        <CardContent className="gap-6">
          <View className="gap-6">
            <View className="gap-1.5">
              <View className="flex-row items-center">
                <Label nativeID="password-label">New password</Label>
              </View>
              <Input
                id="password"
                secureTextEntry
                value={password}
                onChangeText={setPassword}
                returnKeyType="next"
                submitBehavior="submit"
                onSubmitEditing={onPasswordSubmitEditing}
                aria-labelledby="password-label"
              />
            </View>
            <View className="gap-1.5">
              <Label nativeID="code-label">Verification code</Label>
              <Input
                ref={codeInputRef}
                id="code"
                autoCapitalize="none"
                returnKeyType="send"
                keyboardType="numeric"
                autoComplete="sms-otp"
                textContentType="oneTimeCode"
                value={code}
                onChangeText={setCode}
                onSubmitEditing={onSubmit}
                aria-labelledby="code-label"
              />
            </View>
            <Button className="w-full" onPress={onSubmit} disabled={isLoading}>
              <Text>{isLoading ? "Resetting..." : "Reset Password"}</Text>
            </Button>
          </View>
        </CardContent>
      </Card>
    </View>
  );
}
