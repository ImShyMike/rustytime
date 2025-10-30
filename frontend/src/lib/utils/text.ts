import type { UsageStat } from "$lib/types/dashboard";

export function escapeHtml(text: string): string {
    return text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;');
}

export function truncateText(text: string, maxLength: number): string {
    if (maxLength == 0) {
        return text;
    }

    if (text.length <= maxLength) {
        return text;
    }
    return text.slice(0, maxLength - 3) + '...';
}

export function safeText(text: string, maxLength: number = 0): string {
    if (!text) {
        return 'Unknown';
    }
    return escapeHtml(truncateText(text, maxLength));
}

export function safeGraphData(entry: UsageStat[]): UsageStat[] {
    return entry.map((stat) => ({
        ...stat,
        name: safeText(stat.name)
    }));
}