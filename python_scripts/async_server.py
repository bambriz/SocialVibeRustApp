#!/usr/bin/env python3
"""
Social Pulse Python AI Server
HTTP server for sentiment analysis and content moderation using modular architecture
"""
import json
import sys
import os
import time
from http.server import HTTPServer, BaseHTTPRequestHandler

# Import our modular components
from sentiment_analyzer import SentimentAnalyzer
from content_moderator import ContentModerator

class SentimentHandler(BaseHTTPRequestHandler):
    """HTTP request handler for sentiment analysis and content moderation endpoints"""
    
    def do_POST(self):
        try:
            content_length = int(self.headers['Content-Length'])
            post_data = self.rfile.read(content_length)
            data = json.loads(post_data.decode('utf-8'))
            text = data.get('text', '')
            
            if self.path == '/analyze':
                result = sentiment_analyzer.analyze_sentiment(text)
            elif self.path == '/moderate':
                result = content_moderator.moderate_content(text)
            else:
                self.send_error(404)
                return
                
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(json.dumps(result).encode('utf-8'))
            
        except Exception as e:
            print(f"Server error: {e}", file=sys.stderr)
            self.send_error(500, str(e))
    
    def do_GET(self):
        if self.path == '/health':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            
            # Combine status from both modules
            sentiment_status = sentiment_analyzer.get_status()
            moderation_status = content_moderator.get_status()
            
            health_response = {
                "status": "healthy",
                **sentiment_status,
                **moderation_status
            }
            
            self.wfile.write(json.dumps(health_response).encode('utf-8'))
        else:
            self.send_error(404)

def main():
    """Main server initialization and startup"""
    global sentiment_analyzer, content_moderator
    
    print("🚀 Starting Social Pulse Python AI Server with modular architecture...")
    start_time = time.time()
    print("📦 Loading sentiment analysis module...")
    sentiment_analyzer = SentimentAnalyzer()
    
    print("📦 Loading content moderation module...")
    content_moderator = ContentModerator()
    
    init_time = time.time() - start_time
    print(f"⚡ Modules loaded in {init_time:.2f} seconds")
    
    # Start HTTP server
    port = int(os.environ.get('PYTHON_SERVER_PORT', 8001))
    server = HTTPServer(('localhost', port), SentimentHandler)
    print(f"🌐 Server running on http://localhost:{port}")
    print("🔗 Endpoints:")
    print("   POST /analyze  - Sentiment analysis")
    print("   POST /moderate - Content moderation")
    print("   GET  /health   - Health check")
    print("✅ Server ready to accept requests!")
    
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n🛑 Server shutdown requested")
        server.server_close()
        print("✅ Server stopped gracefully")

if __name__ == '__main__':
    main()