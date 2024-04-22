using System.Text;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;

namespace timedpose
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        bool isWindowTopMost = false;
        public MainWindow()
        {
            InitializeComponent();
            Window window = Window.GetWindow(this);
            MainContent.Content = new HomeView();
        }
        private void Window_KeyDown(object sender, KeyEventArgs e)
        {
            // Check if the pressed key is the Enter key
            if (Keyboard.IsKeyDown(Key.LeftShift) || Keyboard.IsKeyDown(Key.RightShift))
            {
                if (e.Key == Key.A)
                {
                    ToggleDisplayWindow();
                }
            }
        }
        private void ToggleDisplayWindow()
        {
            Window window = Window.GetWindow(this);
            if (isWindowTopMost)
            {
                window.Topmost = false;
                isWindowTopMost = false;
            }
            else
            {
                window.Activate();
                window.Topmost = true;
                isWindowTopMost = true;
            }
        }
    }
}