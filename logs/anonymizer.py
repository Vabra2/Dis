#!/usr/bin/env python3
"""
Dis Log Anonymizer
Removes personal data from logs using regex patterns
"""

import re
import hashlib
import os
import logging
from pathlib import Path
from typing import List, Tuple

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class LogAnonymizer:
    """Anonymize logs by removing or hashing personal data"""
    
    # Regex patterns for sensitive data
    PATTERNS = [
        # IP addresses (IPv4)
        (r'\b(?:\d{1,3}\.){3}\d{1,3}\b', 'IP_HASH_{}'),
        # IP addresses (IPv6)
        (r'\b(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\b', 'IPV6_HASH_{}'),
        # Email addresses
        (r'\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b', 'EMAIL_HASH_{}'),
        # JWT tokens (looks like xxx.yyy.zzz)
        (r'\b[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{20,}\b', 'TOKEN_REDACTED'),
        # API keys (common patterns)
        (r'\b[Aa]pi[_-]?[Kk]ey[:\s=]+[A-Za-z0-9_-]{20,}\b', 'API_KEY_REDACTED'),
        # Bearer tokens
        (r'\bBearer\s+[A-Za-z0-9_-]{20,}\b', 'Bearer TOKEN_REDACTED'),
        # User-Agent strings
        (r'User-Agent:\s*.+', 'User-Agent: ANONYMIZED'),
        # Cookie headers
        (r'Cookie:\s*.+', 'Cookie: REDACTED'),
        # Authorization headers
        (r'Authorization:\s*.+', 'Authorization: REDACTED'),
        # X-Forwarded-For
        (r'X-Forwarded-For:\s*.+', 'X-Forwarded-For: ANONYMIZED'),
        # Real IP headers
        (r'X-Real-IP:\s*.+', 'X-Real-IP: ANONYMIZED'),
        # Session IDs
        (r'\bsession[_-]?id[:\s=]+[A-Za-z0-9_-]{20,}\b', 'session_id=REDACTED'),
        # User IDs (but preserve anonymous IDs)
        (r'\buser[_-]?id[:\s=]+(?!anon_)[A-Za-z0-9_-]{10,}\b', 'user_id=REDACTED'),
    ]
    
    def __init__(self, salt: str = "dis-log-salt"):
        """Initialize with salt for hashing"""
        self.salt = salt
        self.hash_cache = {}
    
    def hash_value(self, value: str) -> str:
        """Create consistent hash for a value"""
        if value in self.hash_cache:
            return self.hash_cache[value]
        
        hash_obj = hashlib.sha256()
        hash_obj.update(self.salt.encode())
        hash_obj.update(value.encode())
        hashed = hash_obj.hexdigest()[:8]
        
        self.hash_cache[value] = hashed
        return hashed
    
    def anonymize_line(self, line: str) -> str:
        """Anonymize a single log line"""
        result = line
        
        for pattern, replacement in self.PATTERNS:
            if '{}' in replacement:
                # Hash the matched value
                def replace_with_hash(match):
                    value = match.group(0)
                    hashed = self.hash_value(value)
                    return replacement.format(hashed)
                result = re.sub(pattern, replace_with_hash, result)
            else:
                # Direct replacement
                result = re.sub(pattern, replacement, result)
        
        return result
    
    def anonymize_file(self, input_path: Path, output_path: Path) -> int:
        """Anonymize a log file"""
        lines_processed = 0
        
        try:
            with open(input_path, 'r', encoding='utf-8', errors='ignore') as infile:
                with open(output_path, 'w', encoding='utf-8') as outfile:
                    for line in infile:
                        anonymized = self.anonymize_line(line)
                        outfile.write(anonymized)
                        lines_processed += 1
            
            logger.info(f"Anonymized {lines_processed} lines: {input_path} -> {output_path}")
            return lines_processed
            
        except Exception as e:
            logger.error(f"Error anonymizing {input_path}: {e}")
            return 0
    
    def anonymize_directory(self, directory: Path, in_place: bool = False) -> int:
        """Anonymize all log files in a directory"""
        total_lines = 0
        log_files = list(directory.glob('*.log'))
        
        if not log_files:
            logger.info(f"No log files found in {directory}")
            return 0
        
        for log_file in log_files:
            if in_place:
                # Create temporary file
                temp_file = log_file.with_suffix('.log.tmp')
                lines = self.anonymize_file(log_file, temp_file)
                
                if lines > 0:
                    # Replace original with anonymized version
                    temp_file.replace(log_file)
                    total_lines += lines
                else:
                    # Remove temp file on error
                    temp_file.unlink(missing_ok=True)
            else:
                # Create .anonymized version
                output_file = log_file.with_suffix('.log.anonymized')
                lines = self.anonymize_file(log_file, output_file)
                total_lines += lines
        
        logger.info(f"Total lines anonymized: {total_lines}")
        return total_lines


def watch_and_anonymize(
    log_dir: str,
    interval: int = 60,
    in_place: bool = True
):
    """Continuously watch and anonymize logs"""
    import time
    
    log_path = Path(log_dir)
    anonymizer = LogAnonymizer()
    
    logger.info(f"Starting log anonymizer for {log_path}")
    logger.info(f"Interval: {interval} seconds, In-place: {in_place}")
    
    while True:
        try:
            anonymizer.anonymize_directory(log_path, in_place=in_place)
            time.sleep(interval)
        except KeyboardInterrupt:
            logger.info("Stopping log anonymizer")
            break
        except Exception as e:
            logger.error(f"Error in watch loop: {e}")
            time.sleep(interval)


def main():
    """Main entry point"""
    log_dir = os.getenv('LOG_DIR', '/app/logs')
    interval = int(os.getenv('ANONYMIZE_INTERVAL', '60'))
    in_place = os.getenv('ANONYMIZE_IN_PLACE', 'true').lower() == 'true'
    
    # Ensure log directory exists
    Path(log_dir).mkdir(parents=True, exist_ok=True)
    
    # Run anonymizer
    watch_and_anonymize(log_dir, interval, in_place)


if __name__ == '__main__':
    main()
