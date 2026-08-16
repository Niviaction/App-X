"use client";

import React, { useState } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { FaEye, FaEyeSlash } from 'react-icons/fa';
import '../login/login.css';

const Signup: React.FC = () => {
  const router = useRouter();
  const [email, setEmail] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [password, setPassword] = useState('');
  const [isPasswordVisible, setIsPasswordVisible] = useState(false);

  const handleSignup = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!email || !password) {
      alert('Please enter both your email and password.');
      return;
    }

    const apiUrl = process.env.NEXT_PUBLIC_API_URL;
    if (!apiUrl) {
      console.error('API URL is missing from environment variables.');
      return;
    }

    try {
      const response = await fetch(`${apiUrl}/signup`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        credentials: 'include',
        body: JSON.stringify({
          email,
          password,
          display_name: displayName || undefined,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Sign up failed. Please try again.');
      }

      router.push('/');
    } catch (error) {
      console.error('Error during signup:', error);
      alert(error instanceof Error ? error.message : 'An unexpected error occurred');
    }
  };

  return (
    <div className="login-wrapper">
      <div className="login-card">
        <div className="login-header">
          <h2>Create Your Account</h2>
          <p>Set goals, challenge friends, and stay accountable</p>
        </div>

        <form onSubmit={handleSignup} className="login-form">
          <div className="input-group">
            <input
              type="email"
              placeholder="Email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              autoCapitalize="off"
              autoCorrect="off"
              required
            />
          </div>

          <div className="input-group">
            <input
              type="text"
              placeholder="Display Name (optional)"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
            />
          </div>

          <div className="input-group password-group">
            <input
              type={isPasswordVisible ? 'text' : 'password'}
              placeholder="Password (min. 8 characters)"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              minLength={8}
              required
            />
            <button
              type="button"
              className="eye-button"
              onClick={() => setIsPasswordVisible(!isPasswordVisible)}
              aria-label="Toggle password visibility"
            >
              {isPasswordVisible ? <FaEyeSlash /> : <FaEye />}
            </button>
          </div>

          <button type="submit" className="btn-primary">
            Sign Up
          </button>
        </form>

        <div className="login-footer">
          <p>
            Already have an account? <Link href="/login" className="signup-text">Log In</Link>
          </p>
        </div>
      </div>
    </div>
  );
};

export default Signup;
