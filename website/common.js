// Particle background
const canvas=document.getElementById('particles');
if(canvas){const ctx=canvas.getContext('2d');let P=[];
function R(){canvas.width=window.innerWidth;canvas.height=window.innerHeight}
R();window.addEventListener('resize',R);
class p{constructor(){this.reset();this.y=Math.random()*canvas.height}
reset(){this.x=Math.random()*canvas.width;this.y=-10;this.size=Math.random()*1.5+0.5;this.speed=Math.random()*0.4+0.1;this.opacity=Math.random()*0.5+0.2}
update(){this.y+=this.speed;if(this.y>canvas.height+10)this.reset()}
draw(){ctx.beginPath();ctx.arc(this.x,this.y,this.size,0,Math.PI*2);ctx.fillStyle=`rgba(139,92,246,${this.opacity})`;ctx.fill()}}
for(let i=0;i<120;i++)P.push(new p());
(function A(){ctx.clearRect(0,0,canvas.width,canvas.height);P.forEach(p=>{p.update();p.draw()});requestAnimationFrame(A)})()}

// Nav scroll
window.addEventListener('scroll',()=>{const n=document.getElementById('nav');if(n)n.classList.toggle('scrolled',window.scrollY>50)});

// Scroll reveal
const O=new IntersectionObserver((E)=>{E.forEach(e=>{if(e.isIntersecting)e.target.classList.add('visible')})},{threshold:0.15});
document.querySelectorAll('.reveal').forEach(el=>O.observe(el));

// Highlight active nav link
(function(){const p=location.pathname.split('/').pop()||'index.html';
document.querySelectorAll('.nav-links a').forEach(a=>{if(a.getAttribute('href')===p)a.classList.add('active')})})();