import requests
import base64
import json
from config import GlobalConfig
global_config = GlobalConfig()


class MultiNodeJsonProviderError(Exception):
    pass


class MultiNodeJsonProvider(object):

    def __init__(self):
        nodes = global_config.rpc_url
        best_height = 0
        best_node = None
        for node in nodes:
            self._rpc_addr = node
            node_status = self.ping_node()
            print(node, node_status)
            if not node_status['syncing'] and node_status['latest_block_height'] > best_height + 10:
                best_height = node_status['latest_block_height']
                best_node = node
        if best_node is not None:
            print("Choose near rpc node", best_node)
            self._rpc_addr = best_node
        else:
            raise MultiNodeJsonProviderError("No available nodes")

    def rpc_addr(self):
        return self._rpc_addr

    def json_rpc(self, method, params, timeout=2):
        j = {
            'method': method,
            'params': params,
            'id': 'dontcare',
            'jsonrpc': '2.0'
        }
        r = requests.post(self.rpc_addr(), json=j, timeout=timeout)
        r.raise_for_status()
        content = json.loads(r.content)
        if "error" in content:
            raise MultiNodeJsonProviderError(content["error"])
        return content["result"]

    def get_status(self):
        return self.json_rpc('status', [None])

    def view_call(self, account_id, method_name, args, finality='optimistic'):
        return self.json_rpc('query', {"request_type": "call_function", "account_id": account_id,
                                       "method_name": method_name, "args_base64": base64.b64encode(args).decode('utf8'), "finality": finality})

    def ping_node(self):
        ret = {'latest_block_height': 0, 'syncing': True}

        try:
            status = self.get_status()
            if "sync_info" in status:
                ret['latest_block_height'] = status['sync_info']['latest_block_height']
                ret['syncing'] = status['sync_info']['syncing']
        except MultiNodeJsonProviderError as e:
            print("ping node MultiNodeJsonProviderError: ", e)
        except Exception as e:
            print("ping node Exception: ", e)
    
        return ret


if __name__ == "__main__":
    print("")
