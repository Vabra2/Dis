FROM python:3.11-slim

WORKDIR /app

# Install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application
COPY anonymizer.py .
COPY retention.py .

# Create logs directory
RUN mkdir -p /app/logs

# Run as non-root user
RUN useradd -m -u 1000 loguser && \
    chown -R loguser:loguser /app
USER loguser

# Run both anonymizer and retention (using supervisord in production)
CMD ["python", "anonymizer.py"]
