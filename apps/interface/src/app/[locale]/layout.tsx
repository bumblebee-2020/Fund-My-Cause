import type { Metadata } from "next";
import "../globals.css";
import { WalletProvider } from "@/context/WalletContext";
import { ToastProvider } from "@/components/ui/Toast";
import { ThemeProvider } from "@/context/ThemeContext";
import { NotificationProvider } from "@/context/NotificationContext";
import { ComparisonProvider } from "@/context/ComparisonContext";
import { BookmarkProvider } from "@/context/BookmarkContext";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { ErrorHandlerInitializer } from "@/components/ErrorHandlerInitializer";
import { ModalProvider } from "@/context/ModalContext";
import { SkipNav } from "@/components/ui/SkipNav";
import { NextIntlClientProvider } from "next-intl";
import { getMessages } from "next-intl/server";
import { rtlLocales, type Locale } from "@/i18n/config";
import { notFound } from "next/navigation";
import { routing } from "@/i18n/routing";

export const metadata: Metadata = {
  title: "Fund-My-Cause",
  description: "Decentralized crowdfunding on the Stellar network",
};

export default async function LocaleLayout({
  children,
  params,
}: {
  children: React.ReactNode;
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;

  if (!routing.locales.includes(locale as Locale)) {
    notFound();
  }

  const messages = await getMessages();
  const dir = rtlLocales.includes(locale as Locale) ? "rtl" : "ltr";

  return (
    <html lang={locale} dir={dir} className="dark">
      <body>
        <SkipNav />
        <ErrorBoundary level="page">
          <ErrorHandlerInitializer />
          <NextIntlClientProvider messages={messages}>
            <ThemeProvider>
              <ModalProvider>
                <ToastProvider>
                  <NotificationProvider>
                    <ComparisonProvider>
                      <BookmarkProvider>
                        <WalletProvider>{children}</WalletProvider>
                      </BookmarkProvider>
                    </ComparisonProvider>
                  </NotificationProvider>
                </ToastProvider>
              </ModalProvider>
            </ThemeProvider>
          </NextIntlClientProvider>
        </ErrorBoundary>
      </body>
    </html>
  );
}
