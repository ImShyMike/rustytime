export const creationDateFormatter = new Intl.DateTimeFormat(undefined, {
	dateStyle: 'medium',
	timeStyle: 'short'
});

const relativeTimeFormatter = new Intl.RelativeTimeFormat(undefined, {
	numeric: 'auto'
});

const RELATIVE_DIVISIONS: Array<{ amount: number; unit: Intl.RelativeTimeFormatUnit }> = [
	{ amount: 60, unit: 'second' },
	{ amount: 60, unit: 'minute' },
	{ amount: 24, unit: 'hour' },
	{ amount: 7, unit: 'day' },
	{ amount: 4.34524, unit: 'week' },
	{ amount: 12, unit: 'month' },
	{ amount: Number.POSITIVE_INFINITY, unit: 'year' }
];

export const formatRelativeTime = (from: Date, to = new Date()): string => {
	if (isNaN(from.getTime()) || isNaN(to.getTime())) {
		return 'Unknown';
	}

	let duration = (from.getTime() - to.getTime()) / 1000;

	for (const division of RELATIVE_DIVISIONS) {
		if (Math.abs(duration) < division.amount || division.amount === Number.POSITIVE_INFINITY) {
			return relativeTimeFormatter.format(Math.round(duration), division.unit);
		}

		duration /= division.amount;
	}

	return '';
};

export const formatDuration = (seconds: number): string => {
	if (!Number.isFinite(seconds) || seconds <= 0) {
		return '-';
	}

	const wholeSeconds = Math.floor(seconds);
	const hours = Math.floor(wholeSeconds / 3600);
	const remainingSeconds = wholeSeconds % 3600;
	const minutes = Math.floor(remainingSeconds / 60);
	const secs = remainingSeconds % 60;

	const parts: string[] = [];

	if (hours > 0) {
		parts.push(`${hours}h`);
	}

	if (minutes > 0) {
		parts.push(`${minutes}m`);
	}

	if (parts.length === 0 || secs > 0) {
		parts.push(`${secs}s`);
	}

	return parts.join(' ');
};
