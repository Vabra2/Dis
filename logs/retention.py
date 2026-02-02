#!/usr/bin/env python3
"""
Dis Log Retention Manager
Automatically delete old logs based on retention policy
"""

import os
import logging
from pathlib import Path
from datetime import datetime, timedelta
import time

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class LogRetentionManager:
    """Manage log file retention"""
    
    def __init__(self, retention_days: int):
        """Initialize with retention period"""
        self.retention_days = retention_days
        self.cutoff_date = None
        self.update_cutoff_date()
    
    def update_cutoff_date(self):
        """Update the cutoff date for deletion"""
        self.cutoff_date = datetime.now() - timedelta(days=self.retention_days)
        logger.info(f"Cutoff date set to: {self.cutoff_date}")
    
    def should_delete(self, file_path: Path) -> bool:
        """Check if a file should be deleted based on age"""
        try:
            # Get file modification time
            mtime = datetime.fromtimestamp(file_path.stat().st_mtime)
            return mtime < self.cutoff_date
        except Exception as e:
            logger.error(f"Error checking file {file_path}: {e}")
            return False
    
    def clean_directory(self, directory: Path) -> tuple[int, int]:
        """Clean old log files from directory"""
        deleted_count = 0
        deleted_size = 0
        
        if not directory.exists():
            logger.warning(f"Directory does not exist: {directory}")
            return 0, 0
        
        # Find all log files
        log_patterns = ['*.log', '*.log.gz', '*.log.anonymized']
        log_files = []
        for pattern in log_patterns:
            log_files.extend(directory.glob(pattern))
        
        if not log_files:
            logger.info(f"No log files found in {directory}")
            return 0, 0
        
        # Check each file
        for log_file in log_files:
            if self.should_delete(log_file):
                try:
                    file_size = log_file.stat().st_size
                    log_file.unlink()
                    deleted_count += 1
                    deleted_size += file_size
                    logger.info(f"Deleted old log: {log_file.name} ({file_size} bytes)")
                except Exception as e:
                    logger.error(f"Error deleting {log_file}: {e}")
        
        if deleted_count > 0:
            size_mb = deleted_size / (1024 * 1024)
            logger.info(f"Cleaned {deleted_count} files, freed {size_mb:.2f} MB")
        else:
            logger.info("No old files to delete")
        
        return deleted_count, deleted_size
    
    def clean_all_directories(self, base_dir: Path) -> tuple[int, int]:
        """Recursively clean all directories"""
        total_deleted = 0
        total_size = 0
        
        # Clean base directory
        count, size = self.clean_directory(base_dir)
        total_deleted += count
        total_size += size
        
        # Clean subdirectories
        for subdir in base_dir.iterdir():
            if subdir.is_dir():
                count, size = self.clean_directory(subdir)
                total_deleted += count
                total_size += size
        
        return total_deleted, total_size


def watch_and_clean(
    log_dir: str,
    retention_days: int,
    interval: int = 3600  # Check every hour
):
    """Continuously watch and clean old logs"""
    log_path = Path(log_dir)
    manager = LogRetentionManager(retention_days)
    
    logger.info(f"Starting log retention manager for {log_path}")
    logger.info(f"Retention: {retention_days} days, Check interval: {interval} seconds")
    
    while True:
        try:
            manager.update_cutoff_date()
            total_deleted, total_size = manager.clean_all_directories(log_path)
            
            if total_deleted > 0:
                size_mb = total_size / (1024 * 1024)
                logger.info(f"Total cleanup: {total_deleted} files, {size_mb:.2f} MB freed")
            
            time.sleep(interval)
            
        except KeyboardInterrupt:
            logger.info("Stopping log retention manager")
            break
        except Exception as e:
            logger.error(f"Error in watch loop: {e}")
            time.sleep(interval)


def main():
    """Main entry point"""
    log_dir = os.getenv('LOG_DIR', '/app/logs')
    retention_days = int(os.getenv('LOG_RETENTION_DAYS', '7'))
    check_interval = int(os.getenv('RETENTION_CHECK_INTERVAL', '3600'))
    
    # Ensure log directory exists
    Path(log_dir).mkdir(parents=True, exist_ok=True)
    
    # Run retention manager
    watch_and_clean(log_dir, retention_days, check_interval)


if __name__ == '__main__':
    main()
