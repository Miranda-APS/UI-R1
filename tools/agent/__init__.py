"""Prometeo Semantic Agent - Modulo agente."""
from .ollama_client import OllamaClient, ToolCall, AgentResponse
from .prometeo_bridge import PrometeoBridge, TopologyStats
from .logger import OpinionLogger, OpinionSummarizer, Opinion
from .tools import ToolRegistry, ToolResult
from .loop import PrometeoAgent, LoopConfig, main

__all__ = [
    'OllamaClient', 'ToolCall', 'AgentResponse',
    'PrometeoBridge', 'TopologyStats',
    'OpinionLogger', 'OpinionSummarizer', 'Opinion',
    'ToolRegistry', 'ToolResult',
    'PrometeoAgent', 'LoopConfig', 'main',
]
