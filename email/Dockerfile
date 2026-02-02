FROM python:3.11-slim

WORKDIR /app

# Install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application
COPY sender.py .
COPY templates/ templates/

# Run as non-root user
RUN useradd -m -u 1000 emailuser && \
    chown -R emailuser:emailuser /app
USER emailuser

# Run the service
CMD ["python", "sender.py"]
