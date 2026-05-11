import { FlashList } from "@shopify/flash-list";
import * as Haptics from "expo-haptics";
import { ArrowDownLeft, ArrowUpRight, CheckCircle2, Search, SlidersHorizontal, Users } from "lucide-react-native";
import * as React from "react";
import { View } from "react-native";
import Animated, { FadeIn } from "react-native-reanimated";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Text } from "@/components/ui/text";

const TRANSACTIONS = Array.from({ length: 20 }, (_, i) => ({
  id: i.toString(),
  merchant: i % 3 === 0 ? "Zomato" : i % 3 === 1 ? "Amazon" : "Apple Services",
  date: "Oct 12, 2023",
  amount: i % 2 === 0 ? "-₹1,200.00" : "+₹5,000.00",
  type: i % 2 === 0 ? "OUT" : "IN",
  category: i % 3 === 0 ? "Food" : i % 3 === 1 ? "Shopping" : "Entertainment",
}));

export default function ActivityScreen() {
  const [tabValue, setTabValue] = React.useState("transactions");
  const handlePress = () => {
    Haptics.selectionAsync();
  };

  return (
    <SafeAreaView className="flex-1 bg-background" edges={["top"]}>
      <Animated.View entering={FadeIn} className="flex-1">
        {/* Header */}
        <View className="px-6 py-6 flex-row items-center justify-between">
          <Text className="text-3xl font-bold text-foreground tracking-tight">Activity</Text>
          <View className="flex-row items-center gap-2">
            <Button
              variant="ghost"
              size="icon"
              className="w-10 h-10 rounded-full bg-muted/50"
              onPress={handlePress}
              accessibilityLabel="Search transactions"
            >
              <Search size={20} color="hsl(var(--foreground))" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="w-10 h-10 rounded-full bg-muted/50"
              onPress={handlePress}
              accessibilityLabel="Filter activity"
            >
              <SlidersHorizontal size={20} color="hsl(var(--foreground))" />
            </Button>
          </View>
        </View>

        <Tabs value={tabValue} onValueChange={setTabValue} className="flex-1">
          <View className="px-6 mb-6">
            <TabsList className="bg-muted/50 rounded-2xl p-1.5 flex-row border border-border/40">
              <TabsTrigger value="transactions" className="flex-1 rounded-xl py-2.5" onPress={handlePress}>
                <Text className="font-bold text-xs">Transactions</Text>
              </TabsTrigger>
              <TabsTrigger value="p2p" className="flex-1 rounded-xl py-2.5" onPress={handlePress}>
                <Text className="font-bold text-xs">P2P Splits</Text>
              </TabsTrigger>
              <TabsTrigger value="recon" className="flex-1 rounded-xl py-2.5" onPress={handlePress}>
                <Text className="font-bold text-xs">Recon</Text>
              </TabsTrigger>
            </TabsList>
          </View>

          <View className="flex-1 px-6">
            <TabsContent value="transactions" className="flex-1">
              <FlashList
                data={TRANSACTIONS}
                // @ts-expect-error FlashList typings may be outdated
                estimatedItemSize={90}
                keyExtractor={(item) => item.id}
                showsVerticalScrollIndicator={false}
                contentContainerStyle={{ paddingBottom: 100 }}
                renderItem={({ item }) => (
                  <Card
                    style={{ borderCurve: "continuous" }}
                    className="bg-card/40 border-border/40 rounded-[24px] mb-4 border"
                  >
                    <CardContent className="p-4 flex-row items-center justify-between">
                      <View className="flex-row items-center gap-4">
                        <View
                          className={`w-12 h-12 rounded-2xl items-center justify-center ${item.type === "IN" ? "bg-success-100/50" : "bg-primary/5"}`}
                        >
                          {item.type === "IN" ? (
                            <ArrowDownLeft size={20} color="hsl(var(--success-600))" />
                          ) : (
                            <ArrowUpRight size={20} className="text-primary" />
                          )}
                        </View>
                        <View>
                          <Text className="font-bold text-foreground text-base tracking-tight">{item.merchant}</Text>
                          <View className="flex-row items-center gap-2 mt-0.5">
                            <Text className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">
                              {item.category}
                            </Text>
                            <Text className="text-[10px] text-muted-foreground">•</Text>
                            <Text className="text-[10px] text-muted-foreground font-medium">{item.date}</Text>
                          </View>
                        </View>
                      </View>
                      <Text
                        style={{ fontVariant: ["tabular-nums"] }}
                        className={`font-bold text-base ${item.type === "IN" ? "text-success-600" : "text-foreground"}`}
                      >
                        {item.amount}
                      </Text>
                    </CardContent>
                  </Card>
                )}
              />
            </TabsContent>

            <TabsContent value="p2p" className="flex-1">
              <View
                className="bg-primary rounded-[32px] p-8 mb-8 shadow-lg shadow-primary/20"
                style={{ borderCurve: "continuous" }}
              >
                <View className="flex-row items-center gap-3 mb-4">
                  <View className="w-10 h-10 rounded-full bg-white/10 items-center justify-center">
                    <Users size={20} color="white" />
                  </View>
                  <Text className="font-bold text-white text-xl">Split Requests</Text>
                </View>
                <Text className="text-white/80 mb-6 text-base leading-5">
                  You have 3 split requests from your group waiting for approval.
                </Text>
                <Button className="bg-white rounded-2xl h-14" onPress={handlePress}>
                  <Text className="text-primary font-bold text-base">Review Now</Text>
                </Button>
              </View>

              <Text className="text-muted-foreground font-bold text-xs uppercase tracking-widest mb-4 ml-1">
                Recent Splits
              </Text>

              <FlashList
                data={[1, 2, 3]}
                // @ts-expect-error FlashList typings may be outdated
                estimatedItemSize={90}
                renderItem={({ item: _item }) => (
                  <Card
                    style={{ borderCurve: "continuous" }}
                    className="bg-card/40 border-border/40 rounded-[24px] mb-4 border"
                  >
                    <CardContent className="p-4 flex-row items-center justify-between">
                      <View className="flex-row items-center gap-4">
                        <View className="w-12 h-12 rounded-full bg-muted/80 items-center justify-center">
                          <Text className="font-bold text-muted-foreground">AH</Text>
                        </View>
                        <View>
                          <Text className="font-bold text-foreground text-base">Adrian H.</Text>
                          <Text className="text-xs text-muted-foreground mt-0.5">Sent a split request</Text>
                        </View>
                      </View>
                      <View className="items-end">
                        <Text style={{ fontVariant: ["tabular-nums"] }} className="font-bold text-foreground text-base">
                          ₹450.00
                        </Text>
                        <View className="bg-orange-100/50 px-2 py-0.5 rounded-full mt-1">
                          <Text className="text-[8px] font-bold text-orange-700 uppercase">Pending</Text>
                        </View>
                      </View>
                    </CardContent>
                  </Card>
                )}
              />
            </TabsContent>

            <TabsContent value="recon" className="flex-1">
              <View className="items-center justify-center py-16 px-8 mt-12 bg-muted/20 rounded-[40px] border border-dashed border-border/60">
                <View className="w-24 h-24 bg-success-100 rounded-full items-center justify-center mb-8 shadow-sm">
                  <CheckCircle2 size={48} color="hsl(var(--success-600))" />
                </View>
                <Text className="text-2xl font-bold text-foreground text-center mb-3 tracking-tight">
                  Everything is synced
                </Text>
                <Text className="text-muted-foreground text-center text-base leading-5">
                  Your transactions match your bank statements perfectly.
                </Text>
                <Button
                  variant="outline"
                  className="mt-10 border-border/60 rounded-3xl w-full h-16 bg-background shadow-sm"
                  onPress={handlePress}
                >
                  <Text className="font-bold text-base">Sync Statements</Text>
                </Button>
              </View>
            </TabsContent>
          </View>
        </Tabs>
      </Animated.View>
    </SafeAreaView>
  );
}
