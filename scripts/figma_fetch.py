import os
import requests
import sys
import json

# Ensure utf-8 output for Windows
if sys.stdout.encoding != 'utf-8':
    try:
        import codecs
        sys.stdout = codecs.getwriter('utf-8')(sys.stdout.detach())
    except:
        pass

def fetch_figma_file(token, file_key):
    headers = {
        "X-Figma-Token": token
    }
    url = f"https://api.figma.com/v1/files/{file_key}"
    response = requests.get(url, headers=headers)
    if response.status_code != 200:
        print(f"Error: {response.status_code} - {response.text}", file=sys.stderr)
        sys.exit(1)
    return response.json()

def find_node_by_name(node, name):
    if node.get('name', '').lower() == name.lower():
        return node
    if 'children' in node:
        for child in node['children']:
            result = find_node_by_name(child, name)
            if result:
                return result
    return None

def color_to_hex(color):
    if not color: return "None"
    r = int(color.get('r', 0) * 255)
    g = int(color.get('g', 0) * 255)
    b = int(color.get('b', 0) * 255)
    a = color.get('a', 1)
    return f"#{r:02x}{g:02x}{b:02x} (alpha {a})"

def analyze_fighters_screen(node):
    analysis = []
    analysis.append(f"Analyzing screen: {node.get('name')}")
    
    # Background color of the screen
    bg_color = node.get('backgroundColor')
    analysis.append(f"Screen Background: {color_to_hex(bg_color)}")
    
    if 'children' in node:
        for child in node['children']:
            name = child.get('name', 'Unnamed')
            type = child.get('type', 'Unknown')
            analysis.append(f"\n[{type}] {name}")
            
            # Layout info
            bbox = child.get('absoluteBoundingBox', {})
            analysis.append(f"  Pos: {bbox.get('x')}, {bbox.get('y')} | Size: {bbox.get('width')}x{bbox.get('height')}")
            
            # Colors and effects
            fills = child.get('fills', [])
            for fill in fills:
                if fill.get('type') == 'SOLID':
                    analysis.append(f"  Fill: {color_to_hex(fill.get('color'))}")
            
            effects = child.get('effects', [])
            for effect in effects:
                if effect.get('type') == 'DROP_SHADOW':
                    analysis.append(f"  Shadow: {effect.get('offset', {}).get('x')}, {effect.get('offset', {}).get('y')} blur {effect.get('radius')}")
            
            # Children analysis (simplified)
            if 'children' in child:
                sub_elements = [c.get('name') for c in child['children']]
                analysis.append(f"  Sub-elements: {', '.join(sub_elements[:5])}...")
    
    return "\n".join(analysis)

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python figma_fetch.py <token> <file_key> [node_name]", file=sys.stderr)
        sys.exit(1)
    
    token = sys.argv[1]
    file_key = sys.argv[2]
    node_name = sys.argv[3] if len(sys.argv) > 3 else "fighters"
    
    data = fetch_figma_file(token, file_key)
    
    # Try to find the node
    target_node = find_node_by_name(data['document'], node_name)
    if not target_node:
        target_node = find_node_by_name(data['document'], "Бойцы")
        
    if target_node:
        print(analyze_fighters_screen(target_node))
    else:
        print(f"Node '{node_name}' or 'Бойцы' not found.")
        print("Available pages:")
        for child in data['document']['children']:
            print(f"  - {child.get('name')}")
