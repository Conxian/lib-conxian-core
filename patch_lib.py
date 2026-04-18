import sys

content = open('gateway/src/lib.rs').read()

if 'Engine::poll_support(Arc::clone(&engine)).await;' not in content:
    content = content.replace('Engine::start_monitoring(Arc::clone(&engine)).await;', 'Engine::start_monitoring(Arc::clone(&engine)).await;\n    Engine::poll_support(Arc::clone(&engine)).await;')

with open('gateway/src/lib.rs', 'w') as f:
    f.write(content)
