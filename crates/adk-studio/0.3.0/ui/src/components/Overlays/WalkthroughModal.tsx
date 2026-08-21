/**
 * WalkthroughModal component for ADK Studio v2.0
 * 
 * Provides a guided onboarding experience for new users.
 * Guides through: create project, add agents, connect nodes, run tests.
 * 
 * Requirements: 6.5, 6.6
 */

import { useState } from 'react';
import { X, ChevronRight, ChevronLeft, Check, Sparkles } from 'lucide-react';

/**
 * Walkthrough step definition
 */
interface WalkthroughStep {
  id: string;
  title: string;
  description: string;
  icon: string;
  tips: string[];
}

/**
 * Walkthrough steps for new users
 * Requirements: 6.6
 */
const WALKTHROUGH_STEPS: WalkthroughStep[] = [
  {
    id: 'welcome',
    title: 'Welcome to ADK Studio!',
    description: 'ADK Studio is a visual builder for creating AI agent workflows. Let\'s walk through the basics to get you started.',
    icon: '👋',
    tips: [
      'Build complex agent systems visually',
      'Test and debug in real-time',
      'Export production-ready Rust code',
    ],
  },
  {
    id: 'create-project',
    title: 'Create a Project',
    description: 'Start by creating a new project. Each project contains your agent workflow and configuration.',
    icon: '📁',
    tips: [
      'Click "File → New Project" in the menu',
      'Give your project a descriptive name',
      'Or select a template to start quickly',
    ],
  },
  {
    id: 'add-agents',
    title: 'Add Agents',
    description: 'Drag agents from the left palette onto the canvas. Each agent type has different capabilities.',
    icon: '🤖',
    tips: [
      'LLM Agent: Basic AI agent with model access',
      'Sequential: Run agents in order',
      'Parallel: Run agents simultaneously',
      'Loop: Iterate until a condition is met',
      'Router: Route to different agents based on input',
    ],
  },
  {
    id: 'connect-nodes',
    title: 'Connect Nodes',
    description: 'Connect agents by dragging from one node\'s output handle to another\'s input handle.',
    icon: '🔗',
    tips: [
      'Drag from the bottom handle to the top handle',
      'Double-click an edge to remove it',
      'Use the auto-layout button to organize nodes',
    ],
  },
  {
    id: 'configure-agents',
    title: 'Configure Agents',
    description: 'Click on an agent to open its properties panel. Configure the model, instructions, and tools.',
    icon: '⚙️',
    tips: [
      'Set the system instruction for each agent',
      'Add tools like Google Search or Code Execution',
      'Configure model parameters like temperature',
    ],
  },
  {
    id: 'run-tests',
    title: 'Build & Test',
    description: 'Build your project and test it in the console. Watch agents execute in real-time!',
    icon: '▶️',
    tips: [
      'Click "Build" to compile your workflow',
      'Use the console to send test messages',
      'Watch the timeline to debug execution',
      'Inspect state at each node',
    ],
  },
  {
    id: 'complete',
    title: 'You\'re Ready!',
    description: 'You now know the basics of ADK Studio. Explore templates, experiment with different agent types, and build amazing AI workflows!',
    icon: '🎉',
    tips: [
      'Browse the Template Gallery for inspiration',
      'Export your workflow as Rust code',
      'Check the Help menu for keyboard shortcuts',
    ],
  },
];

interface WalkthroughModalProps {
  /** Callback when walkthrough is completed */
  onComplete: () => void;
  /** Callback when walkthrough is skipped */
  onSkip: () => void;
  /** Callback to close the modal */
  onClose: () => void;
}

/**
 * Walkthrough modal for first-run onboarding
 */
export function WalkthroughModal({ onComplete, onSkip, onClose }: WalkthroughModalProps) {
  const [currentStep, setCurrentStep] = useState(0);
  const step = WALKTHROUGH_STEPS[currentStep];
  const isFirstStep = currentStep === 0;
  const isLastStep = currentStep === WALKTHROUGH_STEPS.length - 1;

  const handleNext = () => {
    if (isLastStep) {
      onComplete();
    } else {
      setCurrentStep(currentStep + 1);
    }
  };

  const handlePrevious = () => {
    if (!isFirstStep) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleSkip = () => {
    onSkip();
  };

  return (
    <div 
      className="fixed inset-0 z-50 flex items-center justify-center p-4"
      style={{ backgroundColor: 'rgba(0, 0, 0, 0.6)' }}
      onClick={onClose}
    >
      <div 
        className="w-full max-w-lg rounded-xl shadow-2xl overflow-hidden"
        style={{ backgroundColor: 'var(--surface-panel)' }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div 
          className="flex items-center justify-between px-6 py-4"
          style={{ 
            backgroundColor: 'var(--accent-primary)',
            color: 'white',
          }}
        >
          <div className="flex items-center gap-2">
            <Sparkles size={20} />
            <span className="font-semibold">Getting Started</span>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-white/20 transition-colors"
          >
            <X size={20} />
          </button>
        </div>

        {/* Progress indicator */}
        <div 
          className="flex gap-1 px-6 py-3"
          style={{ backgroundColor: 'var(--bg-secondary)' }}
        >
          {WALKTHROUGH_STEPS.map((_, index) => (
            <div
              key={index}
              className="flex-1 h-1 rounded-full transition-colors"
              style={{
                backgroundColor: index <= currentStep 
                  ? 'var(--accent-primary)' 
                  : 'var(--border-default)',
              }}
            />
          ))}
        </div>

        {/* Content */}
        <div className="px-6 py-6">
          {/* Step icon and title */}
          <div className="text-center mb-6">
            <span className="text-5xl mb-4 block">{step.icon}</span>
            <h2 
              className="text-xl font-bold mb-2"
              style={{ color: 'var(--text-primary)' }}
            >
              {step.title}
            </h2>
            <p 
              className="text-sm"
              style={{ color: 'var(--text-secondary)' }}
            >
              {step.description}
            </p>
          </div>

          {/* Tips */}
          <div 
            className="rounded-lg p-4 mb-6"
            style={{ backgroundColor: 'var(--bg-secondary)' }}
          >
            <ul className="space-y-2">
              {step.tips.map((tip, index) => (
                <li 
                  key={index}
                  className="flex items-start gap-2 text-sm"
                  style={{ color: 'var(--text-primary)' }}
                >
                  <Check 
                    size={16} 
                    className="mt-0.5 flex-shrink-0"
                    style={{ color: 'var(--accent-primary)' }}
                  />
                  <span>{tip}</span>
                </li>
              ))}
            </ul>
          </div>

          {/* Step counter */}
          <div 
            className="text-center text-xs mb-4"
            style={{ color: 'var(--text-muted)' }}
          >
            Step {currentStep + 1} of {WALKTHROUGH_STEPS.length}
          </div>
        </div>

        {/* Footer with navigation */}
        <div 
          className="flex items-center justify-between px-6 py-4"
          style={{ 
            borderTop: '1px solid var(--border-default)',
            backgroundColor: 'var(--bg-secondary)',
          }}
        >
          <button
            onClick={handleSkip}
            className="px-4 py-2 text-sm rounded transition-colors"
            style={{ color: 'var(--text-secondary)' }}
          >
            Skip Tutorial
          </button>

          <div className="flex items-center gap-2">
            {!isFirstStep && (
              <button
                onClick={handlePrevious}
                className="flex items-center gap-1 px-4 py-2 text-sm rounded transition-colors"
                style={{ 
                  backgroundColor: 'var(--bg-primary)',
                  color: 'var(--text-primary)',
                  border: '1px solid var(--border-default)',
                }}
              >
                <ChevronLeft size={16} />
                Back
              </button>
            )}
            <button
              onClick={handleNext}
              className="flex items-center gap-1 px-4 py-2 text-sm font-medium rounded transition-colors"
              style={{ 
                backgroundColor: 'var(--accent-primary)',
                color: 'white',
              }}
            >
              {isLastStep ? 'Get Started' : 'Next'}
              {!isLastStep && <ChevronRight size={16} />}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
