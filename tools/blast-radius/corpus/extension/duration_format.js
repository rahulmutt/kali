// Format durations and timestamps without a date library: seconds into a human
// string, an ISO-8601 stamp from an epoch value, and a relative "3 hours ago".
// Logs and CLIs need all three and none of them are hard.

const SECONDS_PER = { day: 86400, hour: 3600, minute: 60, second: 1 };
const DAYS_IN_MONTH = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

function pad2(value) {
  return String(value).padStart(2, "0");
}

function isLeapYear(year) {
  return (year % 4 === 0 && year % 100 !== 0) || year % 400 === 0;
}

function humanDuration(totalSeconds) {
  if (totalSeconds < 1) return "under a second";
  let remaining = Math.floor(totalSeconds);
  const parts = [];
  for (const unit of Object.keys(SECONDS_PER)) {
    const size = SECONDS_PER[unit];
    const count = Math.floor(remaining / size);
    if (count === 0) continue;
    remaining -= count * size;
    parts.push(count + " " + unit + (count === 1 ? "" : "s"));
  }
  return parts.slice(0, 2).join(" ");
}

function clockDuration(totalSeconds) {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = Math.floor(totalSeconds % 60);
  return pad2(hours) + ":" + pad2(minutes) + ":" + pad2(seconds);
}

function civilFromEpochDays(days) {
  let year = 1970;
  let remaining = days;
  while (true) {
    const length = isLeapYear(year) ? 366 : 365;
    if (remaining < length) break;
    remaining -= length;
    year += 1;
  }
  let month = 0;
  for (; month < 12; month++) {
    let length = DAYS_IN_MONTH[month];
    if (month === 1 && isLeapYear(year)) length += 1;
    if (remaining < length) break;
    remaining -= length;
  }
  return { year: year, month: month + 1, day: remaining + 1 };
}

function isoStamp(epochSeconds) {
  const days = Math.floor(epochSeconds / 86400);
  const civil = civilFromEpochDays(days);
  const timeOfDay = epochSeconds - days * 86400;
  return (
    civil.year + "-" + pad2(civil.month) + "-" + pad2(civil.day) +
    "T" + clockDuration(timeOfDay) + "Z"
  );
}

function relative(eventEpoch, nowEpoch) {
  const delta = eventEpoch - nowEpoch;
  if (delta === 0) return "just now";
  const text = humanDuration(Math.abs(delta));
  return delta > 0 ? text + " from now" : text + " ago";
}

const NOW = 1755216000;

for (const seconds of [0.4, 45, 90, 3725, 86461, 1209600]) {
  console.log(String(seconds).padStart(8) + "s  " + humanDuration(seconds).padEnd(20) + clockDuration(seconds));
}

for (let stamp of [0, 1000000000, NOW, NOW + 86400 * 200]) {
  console.log(String(stamp).padStart(11) + "  " + isoStamp(stamp));
}

console.log("relative:", relative(NOW - 7200, NOW));
console.log("relative:", relative(NOW + 259200, NOW));
console.log("2024 was a leap year:", isLeapYear(2024));
console.log("1900 was not:", isLeapYear(1900) === false);
