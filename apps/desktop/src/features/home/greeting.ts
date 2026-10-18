export function greetingKey(hour: number): 'home.morning' | 'home.afternoon' | 'home.evening' {
  if (hour < 12) return 'home.morning';
  if (hour < 18) return 'home.afternoon';
  return 'home.evening';
}
